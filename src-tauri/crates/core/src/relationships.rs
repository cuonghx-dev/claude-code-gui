//! Cross-reference extractor for the relationships graph.
//!
//! O(n+m) over agents + commands + skills: a single pass builds reverse
//! indexes so the UI can answer "which commands bind this agent?" or
//! "which agents reference this skill?" in O(1).
//!
//! The extractor is stateless — Phase 5 deliberately re-runs it on every
//! call. Callers that care about latency cache the result and let the
//! frontend `fs:change` listener invalidate the cache (the same pattern
//! agents/commands/skills queries already use).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::AppError;

#[derive(Serialize, Deserialize, TS, Debug, Clone, Default)]
#[ts(export, export_to = "../../../../frontend/src/types/ipc/")]
#[serde(rename_all = "camelCase")]
pub struct RelationshipsGraph {
    /// `agent_slug -> [skill_slug, …]` (skills the agent declares).
    pub agent_skills: BTreeMap<String, Vec<String>>,
    /// `skill_slug -> [agent_slug, …]` (reverse lookup).
    pub skill_agents: BTreeMap<String, Vec<String>>,
    /// `agent_slug -> [command_slug, …]` (commands bound to that agent).
    pub agent_commands: BTreeMap<String, Vec<String>>,
    /// `command_slug -> agent_slug` (forward, single binding).
    pub command_agent: BTreeMap<String, String>,
    /// `skill_slug -> [agent_slug, …]` from skill frontmatter `agent: …`
    /// (a skill can declare a preferred agent the same way commands do).
    pub skill_agent: BTreeMap<String, String>,
}

pub fn build(claude_dir: &Path) -> Result<RelationshipsGraph, AppError> {
    let agents = crate::agents::list(claude_dir)?;
    let commands = crate::commands::list(claude_dir)?;
    let skills = crate::skills::list(claude_dir)?;

    let mut g = RelationshipsGraph::default();

    for a in &agents {
        let mut skill_refs: Vec<String> = a
            .frontmatter
            .skills
            .iter()
            .map(|s| s.to_string())
            .collect();
        skill_refs.sort();
        skill_refs.dedup();
        g.agent_skills.insert(a.slug.clone(), skill_refs.clone());
        for s in skill_refs {
            g.skill_agents
                .entry(s)
                .or_default()
                .push(a.slug.clone());
        }
    }
    for v in g.skill_agents.values_mut() {
        v.sort();
        v.dedup();
    }

    for c in &commands {
        if let Some(agent) = &c.frontmatter.agent {
            g.command_agent.insert(c.slug.clone(), agent.clone());
            g.agent_commands
                .entry(agent.clone())
                .or_default()
                .push(c.slug.clone());
        }
    }
    for v in g.agent_commands.values_mut() {
        v.sort();
        v.dedup();
    }

    for s in &skills {
        if let Some(agent) = &s.frontmatter.agent {
            g.skill_agent.insert(s.slug.clone(), agent.clone());
        }
    }

    Ok(g)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cross_refs_built() {
        let td = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(td.path().join("agents")).unwrap();
        std::fs::create_dir_all(td.path().join("commands")).unwrap();
        std::fs::create_dir_all(td.path().join("skills/refactor")).unwrap();
        std::fs::write(
            td.path().join("agents/reviewer.md"),
            "---\nname: Reviewer\nskills: [refactor]\n---\n\nbody",
        )
        .unwrap();
        std::fs::write(
            td.path().join("commands/review-pr.md"),
            "---\nname: review-pr\nagent: reviewer\n---\n\nbody",
        )
        .unwrap();
        std::fs::write(
            td.path().join("skills/refactor/SKILL.md"),
            "---\nname: refactor\nagent: reviewer\n---\n\nbody",
        )
        .unwrap();
        let g = build(td.path()).unwrap();
        assert_eq!(g.agent_skills["reviewer"], vec!["refactor"]);
        assert_eq!(g.skill_agents["refactor"], vec!["reviewer"]);
        assert_eq!(g.command_agent["review-pr"], "reviewer");
        assert_eq!(g.agent_commands["reviewer"], vec!["review-pr"]);
        assert_eq!(g.skill_agent["refactor"], "reviewer");
    }
}
