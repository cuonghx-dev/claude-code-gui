#!/usr/bin/env bun
// Bump the project version across every place we encode it. Pass the new
// version as the only argument:
//
//     bun scripts/bump-version.ts 0.2.0
//
// Sites updated:
//   - Cargo.toml (workspace.package.version)
//   - frontend/package.json (version)
//   - src-tauri/tauri.conf.json (version)
//
// Lockfiles regenerate via the next `cargo` / `bun install`.

import { readFileSync, writeFileSync } from 'node:fs'

const [, , next] = process.argv
if (!next || !/^\d+\.\d+\.\d+(?:-[\w.]+)?$/.test(next)) {
  console.error('usage: bun scripts/bump-version.ts <semver>')
  process.exit(1)
}

function rewrite(path: string, regex: RegExp, replacement: string) {
  const before = readFileSync(path, 'utf8')
  const after = before.replace(regex, replacement)
  if (before === after) {
    console.warn(`! no version match in ${path}`)
    return
  }
  writeFileSync(path, after)
  console.log(`bumped ${path}`)
}

rewrite('Cargo.toml', /version\s*=\s*"[\d.]+"/, `version    = "${next}"`)
rewrite(
  'frontend/package.json',
  /"version":\s*"[\d.]+(?:-[\w.]+)?"/,
  `"version": "${next}"`,
)
rewrite('src-tauri/tauri.conf.json', /"version":\s*"[\d.]+(?:-[\w.]+)?"/, `"version": "${next}"`)

console.log(`\nnow tag the release: git tag -a v${next} -m 'release ${next}' && git push --tags`)
