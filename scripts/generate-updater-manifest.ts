#!/usr/bin/env bun
// Generate an updater manifest from a directory of release artifacts.
//
// Tauri's updater plugin expects a JSON document of the form:
//   {
//     "version": "0.1.0",
//     "notes": "<release notes>",
//     "pub_date": "<ISO 8601>",
//     "platforms": {
//       "darwin-aarch64": { "signature": "<sig>", "url": "<url>" },
//       "darwin-x86_64":  { ... },
//       "linux-x86_64":   { ... },
//       "windows-x86_64": { ... }
//     }
//   }
//
// We cross-reference each platform's primary archive (.app.tar.gz / .msi.zip /
// .AppImage.tar.gz) with its sibling `.sig` file and emit `updater-manifest.json`
// in the working directory. Asset URLs default to the GitHub Release download
// URL once it's known; we leave them as bare filenames here and the publish
// step rewrites them.

import { readdirSync, readFileSync, writeFileSync, statSync } from 'node:fs'
import { basename, join, relative } from 'node:path'

const [, , bundlesDir = './bundles', version = '0.0.0'] = process.argv

interface Platform {
  signature: string
  url: string
}

function walk(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name)
    if (statSync(p).isDirectory()) walk(p, out)
    else out.push(p)
  }
  return out
}

function platformKey(file: string): string | undefined {
  const name = basename(file)
  if (name.endsWith('aarch64.app.tar.gz') || (name.includes('aarch64') && name.endsWith('.app.tar.gz'))) {
    return 'darwin-aarch64'
  }
  if (name.endsWith('x64.app.tar.gz') || (name.includes('x86_64') && name.endsWith('.app.tar.gz'))) {
    return 'darwin-x86_64'
  }
  if (name.endsWith('.AppImage.tar.gz')) return 'linux-x86_64'
  if (name.endsWith('.msi.zip')) return 'windows-x86_64'
  return undefined
}

const files = walk(bundlesDir)
const platforms: Record<string, Platform> = {}

for (const file of files) {
  const key = platformKey(file)
  if (!key) continue
  const sigPath = `${file}.sig`
  let signature = ''
  try {
    signature = readFileSync(sigPath, 'utf8').trim()
  } catch {
    // No sig file → skip (caller forgot to set TAURI_SIGNING_PRIVATE_KEY).
    continue
  }
  platforms[key] = {
    signature,
    url: relative(bundlesDir, file),
  }
}

const manifest = {
  version: version.replace(/^v/, ''),
  notes: `Release ${version}`,
  pub_date: new Date().toISOString(),
  platforms,
}

writeFileSync('updater-manifest.json', JSON.stringify(manifest, null, 2))
console.log(`wrote updater-manifest.json with ${Object.keys(platforms).length} platforms`)
