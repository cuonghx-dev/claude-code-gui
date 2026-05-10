# ADR 0001 — UI library: radix-vue + Tailwind

**Status**: Accepted
**Date**: 2026-05-10

## Context

SPEC §2 left the UI library decision deferred to the first PR. Options considered: `@nuxt/ui` (Nuxt-only, incompatible with Vite-only setup), `radix-vue` + Tailwind, `Element Plus` / `Vuetify` / `Naive UI` / `PrimeVue` (opinionated component kits with their own theming layer).

## Decision

Adopt **`radix-vue`** for headless primitives (Dialog, Combobox, Popover, Toast, DropdownMenu, Tabs) plus **Tailwind 4** for styling. Auxiliary deps:

- `@tailwindcss/typography` — markdown body rendering
- `lucide-vue-next` — icons (tree-shakeable)
- `class-variance-authority` + `tailwind-merge` — variant-based component styling

## Consequences

- We own visual design end-to-end; no fighting an opinionated theming layer.
- Smaller bundle than full component kits.
- More component code we write ourselves (e.g., custom `<Button>` over importing `<v-btn>`).
- Compatible with shadcn-vue ergonomics if we want to adopt later.

## Rejected alternatives

- **PrimeVue / Element Plus / Vuetify**: ship their own theming layer, balloon bundle by 100–150 KB, fight Tailwind on overrides.
- **shadcn-vue**: viable but commits to a registry workflow and the maintainer's design choices; revisit post-1.0 if ergonomics push that way.
