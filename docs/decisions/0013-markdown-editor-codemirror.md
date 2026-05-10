# ADR 0013 — Markdown editor: CodeMirror 6

**Status**: Accepted
**Date**: 2026-05-10

## Context

Phase 2 forms include a markdown body editor for agents, commands, skills, plans, and output styles. Options: Monaco (heavy, brings worker dance), CodeMirror 6 (smaller, vanilla integration), TipTap / ProseMirror (rich-text with markdown export — complex serialization).

## Decision

CodeMirror 6 with:

- `@codemirror/lang-markdown` — markdown highlighting
- `@codemirror/lang-yaml` — YAML highlighting for the structured frontmatter editor

YAML frontmatter is rendered as a structured form (vee-validate + zod), not raw YAML, so most users never see CodeMirror's YAML mode. It's there for advanced users who switch to "raw" mode.

## Rejected

- **Monaco**: ~2 MB additional bundle, mandatory web worker setup, fights Vue HMR.
- **TipTap**: rich-text → markdown serialization is brittle for the structured frontmatter we expect users to edit.
- **Plain `<textarea>`**: no syntax highlighting, no autocomplete; bad UX for skill bodies that are often hundreds of lines.

## Consequences

- ~250 KB added to the bundle (CodeMirror is modular; only the modes we import ship).
- Forms have a consistent editing experience.
