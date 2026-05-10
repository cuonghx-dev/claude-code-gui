//! Markdown + YAML frontmatter parser.
//!
//! Handles the `---\n<yaml>\n---\n<body>` convention used by every
//! `~/.claude/` markdown file. Round-tripping unknown frontmatter keys is
//! best-effort: `serde_yaml` preserves arbitrary keys via `serde_json::Value`
//! catch-all, but does not preserve key order or comments.

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::AppError;

/// Result of parsing a frontmatter document.
#[derive(Debug, Clone)]
pub struct Document<F> {
    pub frontmatter: F,
    pub body: String,
}

/// Split a markdown source into `(yaml_str, body)`. Returns `(None, src)` if
/// no frontmatter delimiter is found.
fn split(src: &str) -> (Option<&str>, &str) {
    let trimmed = src.strip_prefix('\u{feff}').unwrap_or(src); // strip BOM
    let Some(rest) = trimmed.strip_prefix("---\n").or_else(|| trimmed.strip_prefix("---\r\n")) else {
        return (None, src);
    };
    if let Some(end) = rest.find("\n---\n").or_else(|| rest.find("\r\n---\r\n")) {
        let yaml = &rest[..end];
        let body_start = end + "\n---\n".len();
        let body = &rest[body_start..];
        (Some(yaml), body.trim_start_matches('\n'))
    } else if let Some(end) = rest.find("\n---") {
        // Allow trailing `---` at EOF without body
        let yaml = &rest[..end];
        (Some(yaml), "")
    } else {
        (None, src)
    }
}

pub fn parse<F: DeserializeOwned + Default>(src: &str) -> Result<Document<F>, AppError> {
    let (yaml, body) = split(src);
    let frontmatter = match yaml {
        Some(s) if !s.trim().is_empty() => serde_yaml::from_str(s)?,
        _ => F::default(),
    };
    Ok(Document {
        frontmatter,
        body: body.to_string(),
    })
}

pub fn serialize<F: Serialize>(doc: &Document<F>) -> Result<String, AppError> {
    let yaml = serde_yaml::to_string(&doc.frontmatter)?;
    // serde_yaml emits a leading "---\n" only sometimes; normalize.
    let yaml_clean = yaml.trim_start_matches("---\n").trim_end_matches('\n');
    let body = doc.body.trim_start_matches('\n');
    Ok(format!("---\n{yaml_clean}\n---\n\n{body}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
    struct Fm {
        name: String,
    }

    #[test]
    fn parse_basic() {
        let src = "---\nname: foo\n---\nhello body\n";
        let doc: Document<Fm> = parse(src).unwrap();
        assert_eq!(doc.frontmatter.name, "foo");
        assert_eq!(doc.body, "hello body\n");
    }

    #[test]
    fn parse_no_frontmatter_uses_default() {
        let doc: Document<Fm> = parse("just body").unwrap();
        assert_eq!(doc.frontmatter, Fm::default());
        assert_eq!(doc.body, "just body");
    }

    #[test]
    fn parse_strips_bom() {
        let src = "\u{feff}---\nname: bar\n---\nbody\n";
        let doc: Document<Fm> = parse(src).unwrap();
        assert_eq!(doc.frontmatter.name, "bar");
    }

    #[test]
    fn round_trip_preserves_content() {
        let original = Document {
            frontmatter: Fm { name: "x".into() },
            body: "line1\nline2\n".into(),
        };
        let s = serialize(&original).unwrap();
        let parsed: Document<Fm> = parse(&s).unwrap();
        assert_eq!(parsed.frontmatter, original.frontmatter);
        assert_eq!(parsed.body, original.body);
    }
}
