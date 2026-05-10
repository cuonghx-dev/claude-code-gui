//! Read-side logic for skills. Phase 1: list + get.
//!
//! Local skills: `~/.claude/skills/<slug>/SKILL.md`.
//! Plugin skills: `~/.claude/plugins/<plugin-id>/skills/<slug>/SKILL.md`.

use std::path::Path;

use crate::frontmatter::{self, Document};
use crate::types::{Skill, SkillFrontmatter, SkillSource};
use crate::AppError;

const SKILLS_SUBDIR: &str = "skills";
const PLUGINS_SUBDIR: &str = "plugins";

pub fn list(claude_dir: &Path) -> Result<Vec<Skill>, AppError> {
    let mut out = Vec::new();
    list_into(&claude_dir.join(SKILLS_SUBDIR), SkillSource::Local, &mut out)?;

    let plugins_dir = claude_dir.join(PLUGINS_SUBDIR);
    if plugins_dir.is_dir() {
        for entry in std::fs::read_dir(&plugins_dir)?.flatten() {
            if !entry.path().is_dir() {
                continue;
            }
            let plugin_id = entry.file_name().to_string_lossy().into_owned();
            let plugin_skills = entry.path().join(SKILLS_SUBDIR);
            list_into(&plugin_skills, SkillSource::Plugin { id: plugin_id }, &mut out)?;
        }
    }

    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(out)
}

pub fn get(claude_dir: &Path, slug: &str) -> Result<Skill, AppError> {
    list(claude_dir)?
        .into_iter()
        .find(|s| s.slug == slug)
        .ok_or_else(|| AppError::not_found(format!("skill '{slug}' not found")))
}

fn list_into(root: &Path, source: SkillSource, out: &mut Vec<Skill>) -> Result<(), AppError> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let skill_md = dir.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        match read_one(&dir, &skill_md, source.clone()) {
            Ok(s) => out.push(s),
            Err(e) => tracing::warn!(error = %e, path = %skill_md.display(), "skipping unparseable skill"),
        }
    }
    Ok(())
}

fn read_one(skill_dir: &Path, file: &Path, source: SkillSource) -> Result<Skill, AppError> {
    let src = std::fs::read_to_string(file)?;
    let Document { frontmatter, body }: Document<SkillFrontmatter> = frontmatter::parse(&src)?;
    let slug = skill_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    Ok(Skill {
        slug,
        directory: skill_dir.to_string_lossy().into_owned(),
        frontmatter,
        body,
        file_path: file.to_string_lossy().into_owned(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_and_plugin_skills() {
        let td = tempfile::tempdir().unwrap();
        let local = td.path().join("skills/refactor");
        std::fs::create_dir_all(&local).unwrap();
        std::fs::write(
            local.join("SKILL.md"),
            "---\nname: refactor-helper\ncontext: when\n---\n\nbody\n",
        )
        .unwrap();

        let plugin = td.path().join("plugins/my-plugin/skills/foo");
        std::fs::create_dir_all(&plugin).unwrap();
        std::fs::write(plugin.join("SKILL.md"), "---\nname: foo\n---\n\n").unwrap();

        let skills = list(td.path()).unwrap();
        assert_eq!(skills.len(), 2);
        let plugin_skill = skills.iter().find(|s| s.slug == "foo").unwrap();
        assert!(matches!(&plugin_skill.source, SkillSource::Plugin { id } if id == "my-plugin"));
    }
}
