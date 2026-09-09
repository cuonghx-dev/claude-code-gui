//! Streaming reader + line index for session transcripts
//! (`~/.claude/projects/<encoded-cwd>/<uuid>.jsonl`).
//!
//! Transcripts reach tens of MB. Three features want to walk them — the
//! transcript viewer, usage/cost rollups and file-history checkpoints — so the
//! scan lives here once instead of three `read_to_string` calls.
//!
//! A `LineIndex` maps *message* offsets (what pagination speaks) to *byte*
//! offsets (what `seek` speaks), since one JSONL line expands to zero or more
//! messages. `IndexCache` keeps indexes alive between invocations, keyed on the
//! `(mtime, size)` pair: size alone misses an in-place edit, mtime alone misses
//! a copy that preserves timestamps.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use crate::io::mtime_ms;
use crate::AppError;

/// One transcript line: where it starts, and how many messages precede it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexEntry {
    pub byte_offset: u64,
    pub messages_before: usize,
}

#[derive(Debug, Clone)]
pub struct LineIndex {
    pub entries: Vec<IndexEntry>,
    pub total_messages: usize,
    pub mtime_ms: i64,
    pub size_bytes: u64,
}

impl LineIndex {
    /// Byte offset of the line holding message `index`, plus how many messages
    /// that line starts with. Returns `None` past the end.
    ///
    /// The caller re-parses that whole line and drops the leading
    /// `messages_before` messages — a line yields at most a handful.
    pub fn locate(&self, message_index: usize) -> Option<(u64, usize)> {
        if message_index >= self.total_messages {
            return None;
        }
        // entries are sorted by messages_before; find the last one at or below.
        let pos = match self
            .entries
            .binary_search_by_key(&message_index, |e| e.messages_before)
        {
            Ok(i) => i,
            Err(0) => return None,
            Err(i) => i - 1,
        };
        let entry = self.entries[pos];
        Some((entry.byte_offset, message_index - entry.messages_before))
    }
}

/// Walk a transcript line by line without holding it in memory.
///
/// `f` receives `(byte_offset, line)` and returns `false` to stop early.
/// Lines that fail to read are skipped with a warning rather than aborting the
/// scan — transcripts are appended to live and can hold a torn tail.
pub fn for_each_line<F>(path: &Path, mut f: F) -> Result<(), AppError>
where
    F: FnMut(u64, &str) -> bool,
{
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut offset: u64 = 0;
    let mut buf = String::new();
    loop {
        buf.clear();
        let read = match reader.read_line(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                tracing::warn!(error = %e, path = %path.display(), "stopping transcript scan");
                break;
            }
        };
        let line = buf.trim_end_matches(['\n', '\r']);
        if !line.is_empty() && !f(offset, line) {
            break;
        }
        offset += read as u64;
    }
    Ok(())
}

/// Build a message-to-byte index. `count_messages` reports how many messages a
/// given line parses into — `sessions` owns that knowledge, this module does
/// not.
pub fn build_index<F>(path: &Path, mut count_messages: F) -> Result<LineIndex, AppError>
where
    F: FnMut(&str) -> usize,
{
    let mut entries = Vec::new();
    let mut total = 0usize;
    for_each_line(path, |offset, line| {
        let n = count_messages(line);
        if n > 0 {
            entries.push(IndexEntry {
                byte_offset: offset,
                messages_before: total,
            });
            total += n;
        }
        true
    })?;
    let meta = std::fs::metadata(path)?;
    Ok(LineIndex {
        entries,
        total_messages: total,
        mtime_ms: mtime_ms(path).unwrap_or(0),
        size_bytes: meta.len(),
    })
}

/// Read up to `max_lines` non-empty lines starting at `byte_offset`.
pub fn read_lines_from(
    path: &Path,
    byte_offset: u64,
    max_lines: usize,
) -> Result<Vec<String>, AppError> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(byte_offset))?;
    let mut reader = BufReader::new(file);
    let mut out = Vec::with_capacity(max_lines.min(1024));
    let mut buf = String::new();
    while out.len() < max_lines {
        buf.clear();
        match reader.read_line(&mut buf) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(error = %e, path = %path.display(), "stopping transcript read");
                break;
            }
        }
        let line = buf.trim_end_matches(['\n', '\r']);
        if !line.is_empty() {
            out.push(line.to_string());
        }
    }
    Ok(out)
}

/// Process-wide index cache. Held on `AppState`; entries are dropped when the
/// underlying file's `(mtime, size)` pair changes.
#[derive(Default)]
pub struct IndexCache {
    inner: RwLock<HashMap<PathBuf, Arc<LineIndex>>>,
}

impl IndexCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return a cached index, rebuilding when the file changed underneath it.
    pub fn get_or_build<F>(
        &self,
        path: &Path,
        count_messages: F,
    ) -> Result<Arc<LineIndex>, AppError>
    where
        F: FnMut(&str) -> usize,
    {
        let meta = std::fs::metadata(path)?;
        let (size, mtime) = (meta.len(), mtime_ms(path).unwrap_or(0));

        if let Ok(map) = self.inner.read() {
            if let Some(hit) = map.get(path) {
                if hit.size_bytes == size && hit.mtime_ms == mtime {
                    return Ok(Arc::clone(hit));
                }
            }
        }

        let index = Arc::new(build_index(path, count_messages)?);
        if let Ok(mut map) = self.inner.write() {
            map.insert(path.to_path_buf(), Arc::clone(&index));
        }
        Ok(index)
    }

    /// Drop a single entry. Used when a file is deleted or renamed.
    pub fn forget(&self, path: &Path) {
        if let Ok(mut map) = self.inner.write() {
            map.remove(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every line is one message, except lines containing "skip".
    fn count(line: &str) -> usize {
        if line.contains("skip") {
            0
        } else if line.contains("triple") {
            3
        } else {
            1
        }
    }

    fn fixture(dir: &Path, lines: &[&str]) -> PathBuf {
        let p = dir.join("t.jsonl");
        std::fs::write(&p, format!("{}\n", lines.join("\n"))).unwrap();
        p
    }

    #[test]
    fn index_maps_messages_to_byte_offsets() {
        let td = tempfile::tempdir().unwrap();
        let p = fixture(td.path(), &["{\"a\":1}", "{\"skip\":1}", "{\"triple\":1}", "{\"b\":2}"]);

        let idx = build_index(&p, count).unwrap();
        assert_eq!(idx.total_messages, 5); // 1 + 0 + 3 + 1
        assert_eq!(idx.entries.len(), 3); // the skipped line holds no messages

        // Message 0 is at the very start of the file.
        assert_eq!(idx.locate(0), Some((0, 0)));
        // Messages 1..3 all live on the "triple" line, at increasing sub-offsets.
        let (triple_off, _) = idx.locate(1).unwrap();
        assert_eq!(idx.locate(2), Some((triple_off, 1)));
        assert_eq!(idx.locate(3), Some((triple_off, 2)));
        // Message 4 starts a new line.
        assert!(idx.locate(4).unwrap().0 > triple_off);
        assert_eq!(idx.locate(5), None);
    }

    #[test]
    fn seek_returns_the_same_lines_as_a_full_scan() {
        let td = tempfile::tempdir().unwrap();
        let lines: Vec<String> = (0..50).map(|i| format!("{{\"n\":{i}}}")).collect();
        let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        let p = fixture(td.path(), &refs);

        let idx = build_index(&p, count).unwrap();
        let (offset, _) = idx.locate(30).unwrap();
        let got = read_lines_from(&p, offset, 5).unwrap();
        assert_eq!(got, &lines[30..35]);
    }

    #[test]
    fn for_each_line_can_stop_early() {
        let td = tempfile::tempdir().unwrap();
        let p = fixture(td.path(), &["a", "b", "c", "d"]);
        let mut seen = Vec::new();
        for_each_line(&p, |_, line| {
            seen.push(line.to_string());
            line != "b"
        })
        .unwrap();
        assert_eq!(seen, vec!["a", "b"]);
    }

    #[test]
    fn blank_lines_and_torn_tail_are_skipped() {
        let td = tempfile::tempdir().unwrap();
        let p = td.path().join("t.jsonl");
        // No trailing newline on the last line: a transcript being appended to.
        std::fs::write(&p, "{\"a\":1}\n\n\n{\"b\":2}").unwrap();
        let idx = build_index(&p, count).unwrap();
        assert_eq!(idx.total_messages, 2);
        assert_eq!(read_lines_from(&p, 0, 10).unwrap().len(), 2);
    }

    #[test]
    fn cache_rebuilds_when_the_file_changes() {
        let td = tempfile::tempdir().unwrap();
        let p = fixture(td.path(), &["{\"a\":1}"]);
        let cache = IndexCache::new();

        let first = cache.get_or_build(&p, count).unwrap();
        assert_eq!(first.total_messages, 1);
        // Same file, same stats: the very same Arc comes back.
        let again = cache.get_or_build(&p, count).unwrap();
        assert!(Arc::ptr_eq(&first, &again));

        // Growing the file changes size, so the index is rebuilt.
        std::fs::write(&p, "{\"a\":1}\n{\"b\":2}\n").unwrap();
        let rebuilt = cache.get_or_build(&p, count).unwrap();
        assert_eq!(rebuilt.total_messages, 2);
        assert!(!Arc::ptr_eq(&first, &rebuilt));
    }
}
