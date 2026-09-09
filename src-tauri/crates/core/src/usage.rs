//! Token and cost rollups across every session transcript, plus a prompt
//! activity series from `~/.claude/history.jsonl`.
//!
//! Cost comes from each assistant record's `message.usage`, the only
//! authoritative source. `~/.claude.json` also carries `lastCost` and
//! `lastTotal*` per project, but those describe the *most recent session
//! only*, so they cannot be summed into a history.
//!
//! A full scan of every transcript takes seconds, so per-file turn data is
//! cached under the app's cache directory (never under `~/.claude`, which the
//! watcher subscribes to recursively) and invalidated on the `(mtime, size)`
//! pair. Size alone misses an in-place rewrite; mtime alone misses a copy that
//! preserves timestamps.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::transcript_scan;
use crate::types::{
    ActivityDay, GroupBy, IndexStats, ModelTotals, UsageBucket, UsageQuery, UsageReport,
    UsageTotals,
};
use crate::AppError;

const PROJECTS_SUBDIR: &str = "projects";
const SUBAGENTS_SUBDIR: &str = "subagents";
const HISTORY_FILE: &str = "history.jsonl";
const INDEX_FILE: &str = "usage-index.jsonl";
/// Bump to discard a cache written by an older shape.
const INDEX_VERSION: u32 = 1;

/// One assistant turn's token counts.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TurnUsage {
    pub ts_ms: i64,
    pub model: String,
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    /// Recorded but not yet priced: long-context tiers cost more, and this app
    /// prices only the standard tier for now.
    #[serde(default)]
    pub service_tier: Option<String>,
    #[serde(default)]
    pub is_sidechain: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheEntry {
    pub path: String,
    pub mtime_ms: i64,
    pub size_bytes: u64,
    pub session_id: String,
    pub project_name: String,
    pub turns: Vec<TurnUsage>,
}

#[derive(Serialize, Deserialize, Debug)]
struct IndexHeader {
    v: u32,
}

/// Rescan every transcript whose `(mtime, size)` pair changed, and persist the
/// index. Returns what it had to do, so the UI can say "42 of 900 rescanned".
pub fn refresh_index(claude_dir: &Path, cache_dir: &Path) -> Result<IndexStats, AppError> {
    let started = Instant::now();
    let mut cached = load_index(cache_dir);
    let files = transcript_files(claude_dir)?;

    let mut rescanned = 0usize;
    let mut skipped = 0usize;
    let mut next: Vec<CacheEntry> = Vec::with_capacity(files.len());

    for (path, project_name) in &files {
        let Ok(meta) = std::fs::metadata(path) else {
            continue;
        };
        let size = meta.len();
        let mtime = crate::io::mtime_ms(path).unwrap_or(0);
        let key = path.to_string_lossy().into_owned();

        if let Some(hit) = cached.remove(&key) {
            if hit.size_bytes == size && hit.mtime_ms == mtime {
                next.push(hit);
                skipped += 1;
                continue;
            }
        }
        match scan_file(path, project_name, mtime, size) {
            Ok(entry) => {
                next.push(entry);
                rescanned += 1;
            }
            Err(e) => tracing::warn!(error = %e, path = %path.display(), "usage scan skipped file"),
        }
    }

    save_index(cache_dir, &next)?;
    Ok(IndexStats {
        total_files: files.len(),
        rescanned,
        skipped,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

/// Aggregate the index. Refreshes first, so a caller never reports stale
/// numbers without asking for them.
pub fn rollup(
    claude_dir: &Path,
    cache_dir: &Path,
    q: &UsageQuery,
) -> Result<UsageReport, AppError> {
    refresh_index(claude_dir, cache_dir)?;
    let entries = load_index(cache_dir);

    let mut total = UsageTotals::default();
    let mut buckets: BTreeMap<String, (UsageTotals, HashMap<String, UsageTotals>)> =
        BTreeMap::new();
    let mut unpriced: HashSet<String> = HashSet::new();
    let mut unpriced_turns = 0u64;
    let scanned_files = entries.len();

    for entry in entries.values() {
        if let Some(project) = &q.project {
            if &entry.project_name != project {
                continue;
            }
        }
        for turn in &entry.turns {
            if q.from_ms.is_some_and(|f| turn.ts_ms < f) || q.to_ms.is_some_and(|t| turn.ts_ms > t) {
                continue;
            }
            let cost = crate::models::cost_usd(
                &turn.model,
                turn.input,
                turn.output,
                turn.cache_read,
                turn.cache_write,
            );
            if cost.is_none() && !turn.model.is_empty() {
                unpriced.insert(turn.model.clone());
                unpriced_turns += 1;
            }

            let key = match q.group_by {
                GroupBy::Day => day_key(turn.ts_ms),
                GroupBy::Project => entry.project_name.clone(),
                GroupBy::Model => turn.model.clone(),
            };
            let slot = buckets.entry(key).or_default();
            add(&mut slot.0, turn, cost);
            add(
                slot.1.entry(turn.model.clone()).or_default(),
                turn,
                cost,
            );
            add(&mut total, turn, cost);
        }
    }

    let mut out: Vec<UsageBucket> = buckets
        .into_iter()
        .map(|(key, (totals, by_model))| {
            let mut models: Vec<ModelTotals> = by_model
                .into_iter()
                .map(|(model, totals)| ModelTotals { model, totals })
                .collect();
            models.sort_by(|a, b| b.totals.cost_usd.total_cmp(&a.totals.cost_usd));
            UsageBucket {
                label: bucket_label(&key, q.group_by),
                key,
                totals,
                by_model: models,
            }
        })
        .collect();

    match q.group_by {
        // Days read chronologically; the other groupings are rankings.
        GroupBy::Day => out.sort_by(|a, b| a.key.cmp(&b.key)),
        _ => out.sort_by(|a, b| b.totals.cost_usd.total_cmp(&a.totals.cost_usd)),
    }

    let mut unpriced_models: Vec<String> = unpriced.into_iter().collect();
    unpriced_models.sort();

    Ok(UsageReport {
        total,
        buckets: out,
        scanned_files,
        indexed_at_ms: crate::io::mtime_ms(&cache_dir.join(INDEX_FILE)),
        unpriced_models,
        unpriced_turns,
    })
}

/// Prompt counts per day from `history.jsonl`.
///
/// Counts only. `history.jsonl` holds the raw prompt text, which has no
/// business crossing the IPC boundary for a bar chart.
pub fn activity(claude_dir: &Path, days: u32) -> Result<Vec<ActivityDay>, AppError> {
    let path = claude_dir.join(HISTORY_FILE);
    if !path.is_file() {
        return Ok(vec![]);
    }
    let cutoff_ms = now_ms() - (days as i64) * 86_400_000;
    let mut per_day: BTreeMap<String, (u32, HashSet<String>)> = BTreeMap::new();

    transcript_scan::for_each_line(&path, |_, line| {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            return true;
        };
        let Some(ts) = v.get("timestamp").and_then(|t| t.as_i64()) else {
            return true;
        };
        if ts < cutoff_ms {
            return true;
        }
        let slot = per_day.entry(day_key(ts)).or_default();
        slot.0 += 1;
        if let Some(p) = v.get("project").and_then(|p| p.as_str()) {
            slot.1.insert(p.to_string());
        }
        true
    })?;

    Ok(per_day
        .into_iter()
        .map(|(date, (prompts, projects))| ActivityDay {
            date,
            prompts,
            projects: projects.len() as u32,
        })
        .collect())
}

fn add(t: &mut UsageTotals, turn: &TurnUsage, cost: Option<f64>) {
    t.input += turn.input;
    t.output += turn.output;
    t.cache_read += turn.cache_read;
    t.cache_write += turn.cache_write;
    t.turns += 1;
    t.cost_usd += cost.unwrap_or(0.0);
}

fn bucket_label(key: &str, group_by: GroupBy) -> String {
    match group_by {
        // Project directory names are the path-encoded cwd; the tail is the
        // part a person recognizes.
        GroupBy::Project => key.rsplit('-').next().unwrap_or(key).to_string(),
        _ => key.to_string(),
    }
}

/// Every session transcript, including per-session subagent transcripts —
/// subagent tokens are billed the same as the main thread's.
fn transcript_files(claude_dir: &Path) -> Result<Vec<(PathBuf, String)>, AppError> {
    let root = claude_dir.join(PROJECTS_SUBDIR);
    if !root.is_dir() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for project in std::fs::read_dir(&root)?.flatten() {
        let dir = project.path();
        if !dir.is_dir() {
            continue;
        }
        let name = dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        for entry in std::fs::read_dir(&dir)?.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                out.push((path, name.clone()));
                continue;
            }
            // <session-uuid>/subagents/agent-*.jsonl
            let sub = path.join(SUBAGENTS_SUBDIR);
            if sub.is_dir() {
                for f in std::fs::read_dir(&sub)?.flatten() {
                    let p = f.path();
                    if p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                        out.push((p, name.clone()));
                    }
                }
            }
        }
    }
    Ok(out)
}

fn scan_file(
    path: &Path,
    project_name: &str,
    mtime_ms: i64,
    size_bytes: u64,
) -> Result<CacheEntry, AppError> {
    let mut turns = Vec::new();
    transcript_scan::for_each_line(path, |_, line| {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            return true;
        };
        if v.get("type").and_then(|t| t.as_str()) != Some("assistant") {
            return true;
        }
        let Some(msg) = v.get("message") else {
            return true;
        };
        let Some(u) = msg.get("usage") else {
            return true;
        };
        let n = |key: &str| u.get(key).and_then(|x| x.as_u64()).unwrap_or(0);
        let (input, output) = (n("input_tokens"), n("output_tokens"));
        let (cache_read, cache_write) = (
            n("cache_read_input_tokens"),
            n("cache_creation_input_tokens"),
        );
        if input + output + cache_read + cache_write == 0 {
            return true;
        }
        turns.push(TurnUsage {
            ts_ms: v
                .get("timestamp")
                .and_then(|t| t.as_str())
                .and_then(parse_rfc3339_ms)
                .unwrap_or(0),
            model: msg
                .get("model")
                .and_then(|m| m.as_str())
                .unwrap_or_default()
                .to_string(),
            input,
            output,
            cache_read,
            cache_write,
            service_tier: u
                .get("service_tier")
                .and_then(|s| s.as_str())
                .map(str::to_string),
            is_sidechain: v
                .get("isSidechain")
                .and_then(|s| s.as_bool())
                .unwrap_or(false),
        });
        true
    })?;

    Ok(CacheEntry {
        path: path.to_string_lossy().into_owned(),
        mtime_ms,
        size_bytes,
        session_id: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string(),
        project_name: project_name.to_string(),
        turns,
    })
}

/// Keyed by path. A torn tail line is dropped rather than failing the load —
/// the worst case is one file being rescanned.
fn load_index(cache_dir: &Path) -> HashMap<String, CacheEntry> {
    let path = cache_dir.join(INDEX_FILE);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return HashMap::new();
    };
    let mut lines = raw.lines();
    match lines.next().map(serde_json::from_str::<IndexHeader>) {
        Some(Ok(h)) if h.v == INDEX_VERSION => {}
        // Missing or older header: treat the whole cache as absent.
        _ => return HashMap::new(),
    }
    lines
        .filter_map(|l| serde_json::from_str::<CacheEntry>(l).ok())
        .map(|e| (e.path.clone(), e))
        .collect()
}

fn save_index(cache_dir: &Path, entries: &[CacheEntry]) -> Result<(), AppError> {
    let mut buf = serde_json::to_string(&IndexHeader { v: INDEX_VERSION })?;
    buf.push('\n');
    for e in entries {
        buf.push_str(&serde_json::to_string(e)?);
        buf.push('\n');
    }
    crate::io::atomic_write(&cache_dir.join(INDEX_FILE), buf.as_bytes())
}

fn parse_rfc3339_ms(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

fn day_key(ts_ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ts_ms)
        .map(|dt| dt.format("%Y-%m-%d").to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assistant(ts: &str, model: &str, input: u64, output: u64) -> String {
        format!(
            r#"{{"type":"assistant","timestamp":"{ts}","message":{{"role":"assistant","model":"{model}","content":[{{"type":"text","text":"x"}}],"usage":{{"input_tokens":{input},"output_tokens":{output},"cache_read_input_tokens":0,"cache_creation_input_tokens":0}}}}}}"#
        )
    }

    fn write_session(claude_dir: &Path, project: &str, session: &str, lines: &[String]) -> PathBuf {
        let dir = claude_dir.join(PROJECTS_SUBDIR).join(project);
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join(format!("{session}.jsonl"));
        std::fs::write(&p, lines.join("\n")).unwrap();
        p
    }

    fn query(group_by: GroupBy) -> UsageQuery {
        UsageQuery {
            from_ms: None,
            to_ms: None,
            project: None,
            group_by,
        }
    }

    #[test]
    fn rollup_by_day_sums_turns_and_cost() {
        let td = tempfile::tempdir().unwrap();
        let cache = td.path().join("cache");
        std::fs::create_dir_all(&cache).unwrap();
        write_session(
            td.path(),
            "-tmp-a",
            "s1",
            &[
                assistant("2026-01-01T10:00:00Z", "claude-sonnet-4-6", 1_000_000, 0),
                assistant("2026-01-01T11:00:00Z", "claude-sonnet-4-6", 0, 1_000_000),
                assistant("2026-01-02T10:00:00Z", "claude-sonnet-4-6", 1_000_000, 0),
            ],
        );

        let r = rollup(td.path(), &cache, &query(GroupBy::Day)).unwrap();
        assert_eq!(r.buckets.len(), 2);
        assert_eq!(r.buckets[0].key, "2026-01-01");
        assert_eq!(r.buckets[0].totals.turns, 2);
        // $3/Mtok in + $15/Mtok out on day one.
        assert!((r.buckets[0].totals.cost_usd - 18.0).abs() < 1e-6);
        assert_eq!(r.total.turns, 3);
        assert!(r.unpriced_models.is_empty());
    }

    #[test]
    fn rollup_by_model_and_project_splits() {
        let td = tempfile::tempdir().unwrap();
        let cache = td.path().join("cache");
        std::fs::create_dir_all(&cache).unwrap();
        write_session(
            td.path(),
            "-tmp-a",
            "s1",
            &[assistant("2026-01-01T10:00:00Z", "claude-opus-4-7", 1000, 10)],
        );
        write_session(
            td.path(),
            "-tmp-b",
            "s2",
            &[assistant("2026-01-01T10:00:00Z", "claude-haiku-4-5-20251001", 1000, 10)],
        );

        let by_model = rollup(td.path(), &cache, &query(GroupBy::Model)).unwrap();
        assert_eq!(by_model.buckets.len(), 2);
        // Opus costs more, so it ranks first.
        assert_eq!(by_model.buckets[0].key, "claude-opus-4-7");

        let by_project = rollup(td.path(), &cache, &query(GroupBy::Project)).unwrap();
        assert_eq!(by_project.buckets.len(), 2);

        let mut only_b = query(GroupBy::Project);
        only_b.project = Some("-tmp-b".into());
        let filtered = rollup(td.path(), &cache, &only_b).unwrap();
        assert_eq!(filtered.buckets.len(), 1);
        assert_eq!(filtered.buckets[0].key, "-tmp-b");
    }

    #[test]
    fn unknown_model_is_counted_but_reported_unpriced() {
        let td = tempfile::tempdir().unwrap();
        let cache = td.path().join("cache");
        std::fs::create_dir_all(&cache).unwrap();
        write_session(
            td.path(),
            "-tmp-a",
            "s1",
            &[assistant("2026-01-01T10:00:00Z", "some-other-llm", 1_000_000, 0)],
        );
        let r = rollup(td.path(), &cache, &query(GroupBy::Day)).unwrap();
        assert_eq!(r.total.turns, 1);
        assert_eq!(r.total.cost_usd, 0.0);
        // The number must not read as "$0 spent" without qualification.
        assert_eq!(r.unpriced_models, vec!["some-other-llm"]);
        assert_eq!(r.unpriced_turns, 1);
    }

    #[test]
    fn index_skips_unchanged_files_and_rescans_changed_ones() {
        let td = tempfile::tempdir().unwrap();
        let cache = td.path().join("cache");
        std::fs::create_dir_all(&cache).unwrap();
        let path = write_session(
            td.path(),
            "-tmp-a",
            "s1",
            &[assistant("2026-01-01T10:00:00Z", "claude-sonnet-4-6", 10, 10)],
        );

        let first = refresh_index(td.path(), &cache).unwrap();
        assert_eq!((first.total_files, first.rescanned, first.skipped), (1, 1, 0));

        let second = refresh_index(td.path(), &cache).unwrap();
        assert_eq!((second.rescanned, second.skipped), (0, 1));

        // Appending changes the size, so the file is rescanned.
        let mut body = std::fs::read_to_string(&path).unwrap();
        body.push('\n');
        body.push_str(&assistant("2026-01-01T12:00:00Z", "claude-sonnet-4-6", 10, 10));
        std::fs::write(&path, body).unwrap();

        let third = refresh_index(td.path(), &cache).unwrap();
        assert_eq!((third.rescanned, third.skipped), (1, 0));
        assert_eq!(rollup(td.path(), &cache, &query(GroupBy::Day)).unwrap().total.turns, 2);
    }

    #[test]
    fn torn_index_line_and_stale_version_are_tolerated() {
        let td = tempfile::tempdir().unwrap();
        let cache = td.path().join("cache");
        std::fs::create_dir_all(&cache).unwrap();

        std::fs::write(cache.join(INDEX_FILE), "{\"v\":1}\n{ torn\n").unwrap();
        assert!(load_index(&cache).is_empty());

        std::fs::write(cache.join(INDEX_FILE), "{\"v\":999}\n").unwrap();
        assert!(load_index(&cache).is_empty(), "a newer schema must be discarded");
    }

    #[test]
    fn subagent_transcripts_are_counted() {
        let td = tempfile::tempdir().unwrap();
        let cache = td.path().join("cache");
        std::fs::create_dir_all(&cache).unwrap();
        write_session(
            td.path(),
            "-tmp-a",
            "s1",
            &[assistant("2026-01-01T10:00:00Z", "claude-sonnet-4-6", 10, 10)],
        );
        let sub = td
            .path()
            .join(PROJECTS_SUBDIR)
            .join("-tmp-a")
            .join("s1")
            .join(SUBAGENTS_SUBDIR);
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(
            sub.join("agent-x.jsonl"),
            assistant("2026-01-01T10:05:00Z", "claude-sonnet-4-6", 20, 20),
        )
        .unwrap();

        // Subagent tokens are billed like any other, so they belong in the total.
        let r = rollup(td.path(), &cache, &query(GroupBy::Day)).unwrap();
        assert_eq!(r.total.turns, 2);
        assert_eq!(r.total.input, 30);
    }

    #[test]
    fn activity_counts_prompts_without_exposing_them() {
        let td = tempfile::tempdir().unwrap();
        let now = now_ms();
        let body = format!(
            "{}\n{}\n{}\n",
            format_args!(r#"{{"display":"secret prompt","timestamp":{now},"project":"/a","sessionId":"s"}}"#),
            format_args!(r#"{{"display":"another","timestamp":{now},"project":"/b","sessionId":"s"}}"#),
            format_args!(r#"{{"display":"old","timestamp":1000,"project":"/a","sessionId":"s"}}"#),
        );
        std::fs::write(td.path().join(HISTORY_FILE), body).unwrap();

        let days = activity(td.path(), 30).unwrap();
        assert_eq!(days.len(), 1);
        assert_eq!(days[0].prompts, 2);
        assert_eq!(days[0].projects, 2);
        let wire = serde_json::to_string(&days).unwrap();
        assert!(!wire.contains("secret prompt"), "prompt text crossed the boundary");
    }

    #[test]
    fn empty_claude_dir_yields_an_empty_report() {
        let td = tempfile::tempdir().unwrap();
        let cache = td.path().join("cache");
        std::fs::create_dir_all(&cache).unwrap();
        let r = rollup(td.path(), &cache, &query(GroupBy::Day)).unwrap();
        assert_eq!(r.total.turns, 0);
        assert!(r.buckets.is_empty());
        assert!(activity(td.path(), 30).unwrap().is_empty());
    }
}
