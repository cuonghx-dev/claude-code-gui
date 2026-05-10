# ADR 0014 — Form validation: vee-validate + zod

**Status**: Accepted
**Date**: 2026-05-10

## Context

Phase 2 introduces forms for agent / command / skill / plan / output-style CRUD. We need: typed schemas, async validation (e.g., slug uniqueness via IPC), and field-level error rendering.

## Decision

- `zod` defines typed schemas (matches the frontend's TS-first ergonomic model).
- `vee-validate` binds zod schemas to Vue components via `useForm`/`useField`.

Schemas live alongside their forms (e.g., `frontend/src/components/forms/AgentForm.schema.ts`) and reuse generated `frontend/src/types/ipc/` types as input.

## Rejected

- **`vue-validators` / Vuelidate**: no zod integration; reinvent type bridging.
- **Hand-rolled validation**: doesn't scale beyond two forms.

## Consequences

- Form errors are typed and consistent.
- Schema validation runs both pre-submit (in the form) and post-submit (Rust returns `AppError { code: InvalidInput }` on failures the frontend missed) — defense in depth.
