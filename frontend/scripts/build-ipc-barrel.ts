// Regenerates frontend/src/types/ipc/index.ts as a barrel re-exporting every
// .ts file ts-rs has materialized in that directory. Invoked by the drift
// gate so the barrel itself is part of the diff check.

import { readdirSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'

const DIR = join(import.meta.dir, '..', 'src', 'types', 'ipc')

const files = readdirSync(DIR)
  .filter((f) => f.endsWith('.ts') && f !== 'index.ts' && !f.startsWith('.'))
  .sort()

const lines = [
  '// AUTO-GENERATED. Do not edit by hand.',
  '// Regenerate via `cargo test --workspace` (ts-rs) + `bun run build:ipc-barrel`.',
  '',
  ...files.map((f) => `export * from './${f.replace(/\.ts$/, '')}'`),
  '',
]

writeFileSync(join(DIR, 'index.ts'), lines.join('\n'))
console.log(`wrote ${files.length} re-exports to src/types/ipc/index.ts`)
