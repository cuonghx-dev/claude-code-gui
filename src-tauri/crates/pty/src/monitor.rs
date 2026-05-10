//! Context monitor: parses the PTY output stream for token usage,
//! model identifiers, and tool-call lifecycle markers, and emits
//! `context:tokens:{id}` / `context:tool:{id}` events.
//!
//! Caveat (called out in SPEC §7): regex parsing is brittle to upstream
//! Claude CLI output format changes. Each `claude` upgrade requires a
//! smoke test of the metric panel. The parser is intentionally tolerant
//! — unmatched lines are silently dropped.

use chrono::Utc;
use once_cell::sync::Lazy;
use regex::Regex;

use app_core::types::{TokenUsage, ToolCall};

/// Monitor state for a single session. Survives across chunks because
/// `usage:` lines arrive separately from token deltas.
pub struct MonitorState {
    pub line_buf: String,
    pub last_model: Option<String>,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            line_buf: String::with_capacity(4096),
            last_model: None,
        }
    }
}

/// Parse a raw chunk straight from the PTY reader. Returns a list of
/// events the caller should emit. Original ANSI bytes are not touched —
/// we only strip ANSI internally for regex matching, never for forwarding.
pub fn parse_chunk(state: &mut MonitorState, chunk: &str) -> Vec<MonitorEvent> {
    state.line_buf.push_str(chunk);
    let mut events = Vec::new();

    while let Some(idx) = state.line_buf.find('\n') {
        let raw_line = state.line_buf[..idx].to_string();
        state.line_buf.drain(..=idx);

        let cleaned = String::from_utf8(strip_ansi_escapes::strip(raw_line.as_bytes()))
            .unwrap_or_default();
        let line = cleaned.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(m) = MODEL_RE.captures(line) {
            state.last_model = Some(m[1].to_string());
        }

        if let Some(caps) = TOKEN_RE.captures(line) {
            let usage = TokenUsage {
                input: caps.name("in").and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
                output: caps.name("out").and_then(|m| m.as_str().parse().ok()).unwrap_or(0),
                cache_read: caps
                    .name("cr")
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0),
                cache_write: caps
                    .name("cw")
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0),
            };
            let model = state.last_model.clone();
            let cost = model
                .as_deref()
                .and_then(|alias| compute_cost(alias, &usage))
                .unwrap_or(0.0);
            events.push(MonitorEvent::Tokens {
                usage,
                model,
                cost,
            });
        }

        if let Some(caps) = TOOL_START_RE.captures(line) {
            events.push(MonitorEvent::Tool(ToolCall {
                name: caps[1].to_string(),
                state: "started".into(),
                duration_ms: None,
                timestamp: Utc::now().to_rfc3339(),
            }));
        } else if let Some(caps) = TOOL_DONE_RE.captures(line) {
            events.push(MonitorEvent::Tool(ToolCall {
                name: caps[1].to_string(),
                state: "completed".into(),
                duration_ms: caps.name("ms").and_then(|m| m.as_str().parse().ok()),
                timestamp: Utc::now().to_rfc3339(),
            }));
        }
    }

    events
}

#[derive(Debug)]
pub enum MonitorEvent {
    Tokens {
        usage: TokenUsage,
        model: Option<String>,
        cost: f64,
    },
    Tool(ToolCall),
}

fn compute_cost(model: &str, u: &TokenUsage) -> Option<f64> {
    let m = app_core::models::resolve(model)?;
    Some(
        (u.input as f64 * m.input_price_per_mtok / 1e6)
            + (u.output as f64 * m.output_price_per_mtok / 1e6)
            + (u.cache_read as f64 * m.cache_read_price_per_mtok / 1e6)
            + (u.cache_write as f64 * m.cache_write_price_per_mtok / 1e6),
    )
}

// `tokens: 1234 in, 5678 out, 9 cache_read, 12 cache_write`
static TOKEN_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)tokens:\s*(?P<in>\d+)\s*in.*?(?P<out>\d+)\s*out(?:.*?(?P<cr>\d+)\s*cache_read)?(?:.*?(?P<cw>\d+)\s*cache_write)?",
    )
    .expect("TOKEN_RE")
});

// `usage: ... model=claude-sonnet-4-6` or `model: sonnet`
static MODEL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)model[:=\s]+([A-Za-z0-9._-]+)").expect("MODEL_RE")
});

// `[tool] Grep started` (or `Tool 'Grep' started`)
static TOOL_START_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:\[tool\]\s*|tool\s+'?)([A-Za-z][A-Za-z0-9_-]*)'?\s+started").expect("TOOL_START_RE")
});
// `[tool] Grep completed in 142ms`
static TOOL_DONE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)(?:\[tool\]\s*|tool\s+'?)([A-Za-z][A-Za-z0-9_-]*)'?\s+(?:completed|finished)(?:\s+in\s+(?P<ms>\d+)\s*ms)?",
    )
    .expect("TOOL_DONE_RE")
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_token_line() {
        let mut state = MonitorState::default();
        state.last_model = Some("sonnet".into());
        let events = parse_chunk(
            &mut state,
            "tokens: 100 in, 200 out, 50 cache_read, 10 cache_write\n",
        );
        assert_eq!(events.len(), 1);
        match &events[0] {
            MonitorEvent::Tokens { usage, cost, .. } => {
                assert_eq!(usage.input, 100);
                assert!(*cost > 0.0);
            }
            _ => panic!("expected tokens"),
        }
    }

    #[test]
    fn parses_tool_start_and_done_with_ansi() {
        let mut state = MonitorState::default();
        let chunk = "\x1b[32m[tool]\x1b[0m \x1b[1mGrep\x1b[0m started\n[tool] Grep completed in 142ms\n";
        let events = parse_chunk(&mut state, chunk);
        assert_eq!(events.len(), 2);
        match &events[0] {
            MonitorEvent::Tool(tc) => {
                assert_eq!(tc.name, "Grep");
                assert_eq!(tc.state, "started");
            }
            _ => panic!("expected tool event"),
        }
        match &events[1] {
            MonitorEvent::Tool(tc) => {
                assert_eq!(tc.state, "completed");
                assert_eq!(tc.duration_ms, Some(142));
            }
            _ => panic!("expected tool event"),
        }
    }

    #[test]
    fn buffers_partial_line() {
        let mut state = MonitorState::default();
        let events = parse_chunk(&mut state, "[tool] Grep ");
        assert!(events.is_empty());
        let events = parse_chunk(&mut state, "started\n");
        assert_eq!(events.len(), 1);
    }
}
