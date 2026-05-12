//! Read+write logic for skills.
//!
//! Local skills: `~/.claude/skills/<slug>/SKILL.md`.
//! Plugin skills: `~/.claude/plugins/<plugin-id>/skills/<slug>/SKILL.md`.
//! Only local skills are writable; plugin-bundled skills are read-only.

use std::path::{Path, PathBuf};

use crate::frontmatter::{self, Document};
use crate::io;
use crate::types::{Skill, SkillFrontmatter, SkillImportSource, SkillInput, SkillSource};
use crate::{AppError, ErrorCode};

const SKILLS_SUBDIR: &str = "skills";
const PLUGINS_SUBDIR: &str = "plugins";
const SKILL_FILE: &str = "SKILL.md";

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

pub fn create(claude_dir: &Path, input: SkillInput) -> Result<Skill, AppError> {
    io::validate_slug(&input.slug)?;
    let dir = claude_dir.join(SKILLS_SUBDIR).join(&input.slug);
    if dir.exists() {
        return Err(AppError::invalid(format!(
            "skill '{}' already exists",
            input.slug
        )));
    }
    write_skill(&dir, &input)?;
    get(claude_dir, &input.slug)
}

pub fn update(claude_dir: &Path, slug: &str, input: SkillInput) -> Result<Skill, AppError> {
    io::validate_slug(&input.slug)?;
    let existing = get(claude_dir, slug)?;
    let SkillSource::Local = existing.source else {
        return Err(AppError::invalid(
            "cannot update plugin-bundled skills",
        ));
    };
    let old_dir = PathBuf::from(&existing.directory);
    let new_dir = claude_dir.join(SKILLS_SUBDIR).join(&input.slug);
    if old_dir != new_dir {
        if new_dir.exists() {
            return Err(AppError::invalid(format!(
                "target slug '{}' already exists",
                input.slug
            )));
        }
        std::fs::rename(&old_dir, &new_dir)?;
    }
    write_skill(&new_dir, &input)?;
    get(claude_dir, &input.slug)
}

/// Create a local skill from raw SKILL.md source. Validates frontmatter
/// parses, then writes verbatim.
pub fn create_raw(claude_dir: &Path, slug: &str, content: &str) -> Result<Skill, AppError> {
    io::validate_slug(slug)?;
    let _: Document<SkillFrontmatter> = frontmatter::parse(content)?;
    let dir = claude_dir.join(SKILLS_SUBDIR).join(slug);
    if dir.exists() {
        return Err(AppError::invalid(format!(
            "skill '{slug}' already exists"
        )));
    }
    std::fs::create_dir_all(&dir)?;
    io::atomic_write(&dir.join(SKILL_FILE), content.as_bytes())?;
    get(claude_dir, slug)
}

/// Replace SKILL.md of an existing local skill with `content`. Validates parse.
pub fn update_raw(claude_dir: &Path, slug: &str, content: &str) -> Result<Skill, AppError> {
    let existing = get(claude_dir, slug)?;
    let SkillSource::Local = existing.source else {
        return Err(AppError::invalid("cannot update plugin-bundled skills"));
    };
    let _: Document<SkillFrontmatter> = frontmatter::parse(content)?;
    io::atomic_write(Path::new(&existing.file_path), content.as_bytes())?;
    get(claude_dir, slug)
}

/// Read SKILL.md raw source for use in a markdown editor.
pub fn read_raw(claude_dir: &Path, slug: &str) -> Result<String, AppError> {
    let s = get(claude_dir, slug)?;
    Ok(std::fs::read_to_string(&s.file_path)?)
}

pub fn delete(claude_dir: &Path, slug: &str) -> Result<(), AppError> {
    let existing = get(claude_dir, slug)?;
    let SkillSource::Local = existing.source else {
        return Err(AppError::invalid(
            "cannot delete plugin-bundled skills",
        ));
    };
    io::remove_dir_all(Path::new(&existing.directory))
}

/// Tar+gzip the skill dir and return the archive bytes. Used by the export
/// IPC command which writes those bytes to a user-chosen location.
pub fn export(claude_dir: &Path, slug: &str) -> Result<Vec<u8>, AppError> {
    let skill = get(claude_dir, slug)?;
    let dir = PathBuf::from(&skill.directory);
    tar_gz_dir(&dir).map_err(|e| {
        AppError::new(ErrorCode::Internal, "failed to archive skill directory")
            .with_cause(e)
    })
}

/// Phase 2 implements `Local { path }`; `Github { url }` returns `Internal`
/// and lands in Phase 3 (network fetch + extract).
pub fn import(claude_dir: &Path, source: SkillImportSource) -> Result<Vec<Skill>, AppError> {
    match source {
        SkillImportSource::Local { path } => {
            let src = PathBuf::from(&path);
            if !src.is_dir() {
                return Err(AppError::invalid(format!(
                    "skill source is not a directory: {path}"
                )));
            }
            let skill_md = src.join(SKILL_FILE);
            if !skill_md.is_file() {
                return Err(AppError::invalid(format!(
                    "no SKILL.md in '{path}'"
                )));
            }
            let slug = src
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| AppError::invalid("source path has no name"))?
                .to_string();
            io::validate_slug(&slug)?;
            let dest_dir = claude_dir.join(SKILLS_SUBDIR).join(&slug);
            if dest_dir.exists() {
                return Err(AppError::invalid(format!(
                    "skill '{slug}' already exists"
                )));
            }
            copy_dir_recursive(&src, &dest_dir)?;
            Ok(vec![get(claude_dir, &slug)?])
        }
        SkillImportSource::Github { .. } => Err(AppError::new(
            ErrorCode::Internal,
            "github skill import lands in Phase 3",
        )),
    }
}

fn write_skill(dir: &Path, input: &SkillInput) -> Result<(), AppError> {
    std::fs::create_dir_all(dir)?;
    let doc = Document {
        frontmatter: input.frontmatter.clone(),
        body: input.body.clone(),
    };
    let serialized = frontmatter::serialize(&doc)?;
    io::atomic_write(&dir.join(SKILL_FILE), serialized.as_bytes())
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<(), AppError> {
    std::fs::create_dir_all(to)?;
    for entry in walkdir::WalkDir::new(from) {
        let entry = entry.map_err(|e| AppError::new(ErrorCode::IoError, e.to_string()))?;
        let rel = entry
            .path()
            .strip_prefix(from)
            .map_err(|e| AppError::new(ErrorCode::Internal, e.to_string()))?;
        let dest = to.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else if entry.file_type().is_file() {
            if let Some(p) = dest.parent() {
                std::fs::create_dir_all(p)?;
            }
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

/// Minimal hand-rolled tar (POSIX ustar, no compression). Adequate for skill
/// dirs (a handful of small files); we deliberately avoid pulling in `tar`
/// or `flate2` deps for Phase 2's narrow use case.
fn tar_gz_dir(dir: &Path) -> std::io::Result<Vec<u8>> {
    use std::io::{Read, Write};

    let mut tar = Vec::<u8>::new();
    let base_name = dir
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.map_err(std::io::Error::other)?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(dir)
            .map_err(std::io::Error::other)?;
        let name = format!("{base_name}/{}", rel.to_string_lossy());

        let mut f = std::fs::File::open(entry.path())?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        // ustar header (512 bytes).
        let mut header = [0u8; 512];
        write_field(&mut header[0..100], &name);
        write_field(&mut header[100..108], "0000644 ");
        write_field(&mut header[108..116], "0000000 ");
        write_field(&mut header[116..124], "0000000 ");
        write_field(&mut header[124..136], &format!("{:011o} ", buf.len()));
        write_field(&mut header[136..148], &format!("{:011o} ", 0));
        // checksum placeholder: 8 spaces during compute.
        for b in &mut header[148..156] {
            *b = b' ';
        }
        header[156] = b'0'; // typeflag = regular file
        write_field(&mut header[257..263], "ustar ");
        write_field(&mut header[263..265], " ");
        let checksum: u32 = header.iter().map(|b| *b as u32).sum();
        let cs = format!("{checksum:06o}\0 ");
        header[148..148 + cs.len()].copy_from_slice(cs.as_bytes());

        tar.write_all(&header)?;
        tar.write_all(&buf)?;
        let pad = (512 - (buf.len() % 512)) % 512;
        tar.extend(std::iter::repeat(0u8).take(pad));
    }
    // two zero blocks terminate the archive.
    tar.extend(std::iter::repeat(0u8).take(1024));

    Ok(tar)
}

fn write_field(field: &mut [u8], src: &str) {
    let bytes = src.as_bytes();
    let n = bytes.len().min(field.len());
    field[..n].copy_from_slice(&bytes[..n]);
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

    #[test]
    fn create_update_delete() {
        let td = tempfile::tempdir().unwrap();
        let input = SkillInput {
            slug: "alpha".into(),
            frontmatter: SkillFrontmatter {
                name: Some("Alpha".into()),
                ..Default::default()
            },
            body: "alpha body".into(),
        };
        create(td.path(), input).unwrap();
        assert!(get(td.path(), "alpha").is_ok());

        let renamed = update(
            td.path(),
            "alpha",
            SkillInput {
                slug: "beta".into(),
                frontmatter: SkillFrontmatter::default(),
                body: "b".into(),
            },
        )
        .unwrap();
        assert_eq!(renamed.slug, "beta");

        delete(td.path(), "beta").unwrap();
        assert!(get(td.path(), "beta").is_err());
    }

    #[test]
    fn import_local_copies_dir() {
        let td = tempfile::tempdir().unwrap();
        let src_dir = td.path().join("external/my-skill");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(
            src_dir.join("SKILL.md"),
            "---\nname: Imported\n---\n\nbody\n",
        )
        .unwrap();
        std::fs::write(src_dir.join("extra.txt"), "extra").unwrap();

        let claude_dir = td.path().join("home");
        std::fs::create_dir_all(&claude_dir).unwrap();

        let imported = import(
            &claude_dir,
            SkillImportSource::Local {
                path: src_dir.to_string_lossy().into_owned(),
            },
        )
        .unwrap();
        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].slug, "my-skill");
        assert!(claude_dir.join("skills/my-skill/extra.txt").is_file());
    }

    #[test]
    fn export_returns_tar_with_skill_md() {
        let td = tempfile::tempdir().unwrap();
        create(
            td.path(),
            SkillInput {
                slug: "exp".into(),
                frontmatter: SkillFrontmatter::default(),
                body: "body".into(),
            },
        )
        .unwrap();
        let bytes = export(td.path(), "exp").unwrap();
        assert!(bytes.len() > 1024);
        // Filename in tar header should appear.
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("exp/SKILL.md"));
    }
}
