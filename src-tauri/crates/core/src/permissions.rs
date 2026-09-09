//! The `permissions` block of a settings file: allow/ask/deny rules,
//! additional directories, and the mode a session starts in.
//!
//! Two facts from the permissions docs shape this module:
//!
//! * Rule lists **merge across scopes** — a rule in user settings still
//!   applies when the project file also sets `permissions.allow`. `effective`
//!   unions them and tags each rule with the file it came from.
//! * Rule syntax is `Tool` or `Tool(specifier)`, with per-tool specifier
//!   conventions (`Bash(npm run *)`, `Read(./.env)`,
//!   `WebFetch(domain:example.com)`, `mcp__server__tool`).
//!
//! Validation warns rather than rejects for anything but a malformed rule. The
//! CLI adds tools and modes faster than this app can follow, and a rule this
//! app does not recognize is far more likely to be new than wrong.

use std::path::Path;

use serde_json::{json, Value};

use crate::settings_scope;
use crate::types::{
    EffectivePermissions, PermissionRule, Permissions, RuleIssue, RuleKind, ScopedRule, Severity,
    SettingsScope,
};
use crate::AppError;

/// Modes documented for `permissions.defaultMode`. Anything else warns.
const KNOWN_MODES: &[&str] = &[
    "default",
    "manual", // documented alias for `default`
    "acceptEdits",
    "plan",
    "auto",
    "dontAsk",
    "bypassPermissions",
];

const PATH_TOOLS: &[&str] = &["Read", "Edit", "Write", "Glob", "Grep", "NotebookEdit"];
const WEB_TOOLS: &[&str] = &["WebFetch", "WebSearch"];

pub fn get(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
) -> Result<Permissions, AppError> {
    let path = settings_scope::scope_path(claude_dir, scope, working_dir)?;
    let doc = crate::io::read_json_doc(&path)?;
    Ok(doc
        .value
        .get("permissions")
        .cloned()
        .map(|v| serde_json::from_value(v).unwrap_or_default())
        .unwrap_or_default())
}

pub fn put(
    claude_dir: &Path,
    scope: SettingsScope,
    working_dir: Option<&Path>,
    perms: &Permissions,
    expected_mtime_ms: Option<i64>,
) -> Result<i64, AppError> {
    // Serialize through the typed struct but patch only the `permissions` key,
    // so every sibling setting is left exactly as it was.
    let mut value = serde_json::to_value(perms)?;
    if let Some(obj) = value.as_object_mut() {
        // An empty list is indistinguishable from "not set" to Claude Code, and
        // leaving empties behind makes the file noisier every save.
        obj.retain(|_, v| !matches!(v, Value::Array(a) if a.is_empty()));
    }
    settings_scope::patch(
        claude_dir,
        scope,
        working_dir,
        &json!({ "permissions": value }),
        expected_mtime_ms,
    )
}

/// Every rule that applies, tagged with the file that contributes it.
pub fn effective(
    claude_dir: &Path,
    working_dir: Option<&Path>,
) -> Result<EffectivePermissions, AppError> {
    let mut out = EffectivePermissions {
        allow: vec![],
        ask: vec![],
        deny: vec![],
        additional_directories: vec![],
        default_mode: None,
        default_mode_source: None,
    };

    // Highest precedence first: scalars take the first value seen, lists take
    // everything.
    for &scope in SettingsScope::ALL.iter() {
        let Ok(p) = get(claude_dir, scope, working_dir) else {
            continue;
        };
        let push = |dest: &mut Vec<ScopedRule>, rules: &[String]| {
            for rule in rules {
                if !dest.iter().any(|r| &r.rule == rule) {
                    dest.push(ScopedRule {
                        rule: rule.clone(),
                        scope,
                    });
                }
            }
        };
        push(&mut out.allow, &p.allow);
        push(&mut out.ask, &p.ask);
        push(&mut out.deny, &p.deny);
        push(&mut out.additional_directories, &p.additional_directories);

        if out.default_mode.is_none() {
            if let Some(mode) = p.default_mode {
                out.default_mode = Some(mode);
                out.default_mode_source = Some(scope);
            }
        }
    }
    Ok(out)
}

/// Parse `Tool` or `Tool(specifier)`.
pub fn parse_rule(s: &str) -> Result<PermissionRule, AppError> {
    let raw = s.trim();
    if raw.is_empty() {
        return Err(AppError::invalid("rule cannot be empty"));
    }

    let (tool, specifier) = match raw.find('(') {
        Some(open) => {
            if !raw.ends_with(')') {
                return Err(AppError::invalid(format!("unbalanced parentheses in '{raw}'")));
            }
            let tool = &raw[..open];
            let spec = &raw[open + 1..raw.len() - 1];
            if spec.is_empty() {
                return Err(AppError::invalid(format!("empty specifier in '{raw}'")));
            }
            (tool, Some(spec.to_string()))
        }
        None => (raw, None),
    };

    if tool.is_empty() {
        return Err(AppError::invalid(format!("missing tool name in '{raw}'")));
    }
    let valid_tool = tool.starts_with("mcp__")
        || tool
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !valid_tool {
        return Err(AppError::invalid(format!("invalid tool name '{tool}'")));
    }

    let kind = if tool.starts_with("mcp__") {
        RuleKind::Mcp
    } else if specifier.is_none() {
        RuleKind::Bare
    } else if tool == "Bash" {
        RuleKind::Bash
    } else if PATH_TOOLS.contains(&tool) {
        RuleKind::PathLike
    } else if WEB_TOOLS.contains(&tool) {
        RuleKind::Domain
    } else {
        RuleKind::Other
    };

    Ok(PermissionRule {
        raw: raw.to_string(),
        tool: tool.to_string(),
        specifier,
        kind,
    })
}

/// Check a set of rules. Errors are malformed syntax; everything else is a
/// warning, because the tool and mode vocabularies keep growing.
pub fn validate(perms: &Permissions) -> Vec<RuleIssue> {
    let mut issues = Vec::new();

    for (list, label) in [
        (&perms.allow, "allow"),
        (&perms.ask, "ask"),
        (&perms.deny, "deny"),
    ] {
        let mut seen: Vec<&String> = Vec::new();
        for rule in list {
            match parse_rule(rule) {
                Err(e) => issues.push(RuleIssue {
                    rule: rule.clone(),
                    severity: Severity::Error,
                    message: e.message,
                }),
                Ok(parsed) => {
                    if parsed.kind == RuleKind::Domain
                        && parsed
                            .specifier
                            .as_deref()
                            .is_some_and(|s| !s.starts_with("domain:"))
                    {
                        issues.push(RuleIssue {
                            rule: rule.clone(),
                            severity: Severity::Warning,
                            message: format!(
                                "{} specifiers are usually written as domain:example.com",
                                parsed.tool
                            ),
                        });
                    }
                }
            }
            if seen.contains(&rule) {
                issues.push(RuleIssue {
                    rule: rule.clone(),
                    severity: Severity::Warning,
                    message: format!("duplicate rule in {label}"),
                });
            }
            seen.push(rule);
        }
    }

    // deny beats allow at evaluation time, so a rule in both is dead weight the
    // author probably did not intend.
    for rule in &perms.allow {
        if perms.deny.contains(rule) {
            issues.push(RuleIssue {
                rule: rule.clone(),
                severity: Severity::Warning,
                message: "rule is in both allow and deny; deny wins".into(),
            });
        }
    }

    if let Some(mode) = &perms.default_mode {
        if !KNOWN_MODES.contains(&mode.as_str()) {
            issues.push(RuleIssue {
                rule: mode.clone(),
                severity: Severity::Warning,
                message: format!(
                    "'{mode}' is not a mode this version knows about; it may be newer than this app"
                ),
            });
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn perms(allow: &[&str], deny: &[&str]) -> Permissions {
        Permissions {
            allow: allow.iter().map(|s| s.to_string()).collect(),
            deny: deny.iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn parses_the_documented_rule_shapes() {
        let cases = [
            ("Bash", "Bash", None, RuleKind::Bare),
            ("Bash(npm run build)", "Bash", Some("npm run build"), RuleKind::Bash),
            ("Bash(git diff:*)", "Bash", Some("git diff:*"), RuleKind::Bash),
            ("Read(./.env)", "Read", Some("./.env"), RuleKind::PathLike),
            ("Read(//**/.env)", "Read", Some("//**/.env"), RuleKind::PathLike),
            (
                "WebFetch(domain:example.com)",
                "WebFetch",
                Some("domain:example.com"),
                RuleKind::Domain,
            ),
            (
                "mcp__github__create_issue",
                "mcp__github__create_issue",
                None,
                RuleKind::Mcp,
            ),
        ];
        for (raw, tool, spec, kind) in cases {
            let r = parse_rule(raw).unwrap_or_else(|e| panic!("{raw}: {}", e.message));
            assert_eq!(r.tool, tool, "{raw}");
            assert_eq!(r.specifier.as_deref(), spec, "{raw}");
            assert_eq!(r.kind, kind, "{raw}");
        }
    }

    #[test]
    fn rejects_malformed_rules() {
        for bad in ["", "   ", "Bash(", "()", "(x)", "Bad Tool(x)"] {
            assert!(parse_rule(bad).is_err(), "expected '{bad}' to fail");
        }
        // A nested paren inside the specifier is fine — commands contain them.
        assert_eq!(
            parse_rule("Bash(echo $(date))").unwrap().specifier.as_deref(),
            Some("echo $(date)")
        );
    }

    #[test]
    fn validation_warns_but_does_not_reject_unknown_vocabulary() {
        let mut p = perms(&["Bash(ls:*)", "SomeNewTool(x)"], &[]);
        // "auto" is a real mode; a made-up one only warns.
        p.default_mode = Some("auto".into());
        assert!(validate(&p).is_empty(), "auto is a documented mode");

        p.default_mode = Some("turboMode".into());
        let issues = validate(&p);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Warning);
    }

    #[test]
    fn validation_flags_duplicates_and_allow_deny_overlap() {
        let p = perms(&["Bash(ls:*)", "Bash(ls:*)", "Read(a)"], &["Read(a)"]);
        let issues = validate(&p);
        assert!(issues.iter().any(|i| i.message.contains("duplicate")));
        assert!(issues
            .iter()
            .any(|i| i.message.contains("both allow and deny")));
        assert!(issues.iter().all(|i| i.severity == Severity::Warning));
    }

    #[test]
    fn malformed_rule_is_an_error_not_a_warning() {
        let p = perms(&["Bash("], &[]);
        let issues = validate(&p);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, Severity::Error);
    }

    #[test]
    fn put_preserves_sibling_settings_and_unknown_permission_keys() {
        let td = tempfile::tempdir().unwrap();
        let path = td.path().join("settings.json");
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&json!({
                "model": "opus",
                "hooks": {"PreToolUse": []},
                "permissions": {"allow": ["A"], "someFuturePermissionKey": true}
            }))
            .unwrap(),
        )
        .unwrap();

        let mut p = get(td.path(), SettingsScope::User, None).unwrap();
        assert!(p.extra.contains_key("someFuturePermissionKey"));
        p.allow.push("Bash(ls:*)".into());
        put(td.path(), SettingsScope::User, None, &p, None).unwrap();

        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["model"], "opus");
        assert!(v["hooks"].is_object());
        assert_eq!(v["permissions"]["allow"], json!(["A", "Bash(ls:*)"]));
        assert_eq!(v["permissions"]["someFuturePermissionKey"], true);
        // Empty lists are not written back as noise.
        assert!(v["permissions"].get("deny").is_none());
    }

    #[test]
    fn effective_unions_rules_across_scopes_and_tags_their_source() {
        let td = tempfile::tempdir().unwrap();
        let wd = td.path().join("proj");
        std::fs::create_dir_all(wd.join(".claude")).unwrap();
        std::fs::write(
            td.path().join("settings.json"),
            r#"{"permissions":{"allow":["Read(a)"],"defaultMode":"plan"}}"#,
        )
        .unwrap();
        std::fs::write(
            wd.join(".claude/settings.json"),
            r#"{"permissions":{"allow":["Read(b)"],"deny":["Bash(rm *)"],"defaultMode":"acceptEdits"}}"#,
        )
        .unwrap();

        let eff = effective(td.path(), Some(&wd)).unwrap();
        assert_eq!(eff.allow.len(), 2, "rules merge across scopes");
        let a = eff.allow.iter().find(|r| r.rule == "Read(a)").unwrap();
        assert_eq!(a.scope, SettingsScope::User);
        let b = eff.allow.iter().find(|r| r.rule == "Read(b)").unwrap();
        assert_eq!(b.scope, SettingsScope::Project);
        assert_eq!(eff.deny.len(), 1);
        // A scalar follows precedence instead of merging.
        assert_eq!(eff.default_mode.as_deref(), Some("acceptEdits"));
        assert_eq!(eff.default_mode_source, Some(SettingsScope::Project));
    }

    #[test]
    fn missing_permissions_block_reads_as_empty() {
        let td = tempfile::tempdir().unwrap();
        let p = get(td.path(), SettingsScope::User, None).unwrap();
        assert!(p.allow.is_empty() && p.default_mode.is_none());
        assert!(validate(&p).is_empty());
    }
}
