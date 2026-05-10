// Frontend-only UI registry: label, color, tagline. Pricing and context
// windows live in Rust (`crates/core/src/models.rs`) and reach the frontend
// via ts-rs generated `ModelMeta`. Two registries on purpose — see SPEC §3.

export const MODEL = {
  OPUS: 'opus',
  SONNET: 'sonnet',
  HAIKU: 'haiku',
} as const

export type ModelAlias = (typeof MODEL)[keyof typeof MODEL]

export const MODEL_IDS: ModelAlias[] = [MODEL.OPUS, MODEL.SONNET, MODEL.HAIKU]

export const MODEL_META: Record<ModelAlias, { label: string; color: string; tagline: string }> = {
  [MODEL.OPUS]:   { label: 'Opus 4.7',    color: '#7c3aed', tagline: 'Most capable. Best for complex reasoning.' },
  [MODEL.SONNET]: { label: 'Sonnet 4.6',  color: '#0ea5e9', tagline: 'Balanced. Default for most agents.' },
  [MODEL.HAIKU]:  { label: 'Haiku 4.5',   color: '#10b981', tagline: 'Fast & cheap. Good for tight loops.' },
}
