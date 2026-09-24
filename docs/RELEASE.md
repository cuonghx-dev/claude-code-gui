# Release pipeline

> **Status:** the GitHub Actions workflows (`ci.yml`, `release.yml`) were
> removed in `82da7d3`. Until they are restored (`git show 82da7d3^:.github/workflows/release.yml`),
> releases are built locally with the commands in [Building a release locally](#building-a-release-locally).
> The rest of this document describes the pipeline those workflows ran.

## Overview

`.github/workflows/release.yml` is triggered by pushing a `v*.*.*` tag. It
builds bundles for four targets, signs/notarizes them on the appropriate
platform, generates an updater manifest, and publishes a GitHub Release.

| Target | Bundle artifacts | Signing |
|--------|------------------|---------|
| `aarch64-apple-darwin` | `.dmg` + `.app.tar.gz`(+`.sig`) | Developer ID + notarytool |
| `x86_64-apple-darwin`  | `.dmg` + `.app.tar.gz`(+`.sig`) | Developer ID + notarytool |
| `x86_64-pc-windows-msvc` | `.msi` + `.msi.zip`(+`.sig`) | Authenticode (EV cert via signtool) |
| `x86_64-unknown-linux-gnu` | `.AppImage` + `.deb` + `.rpm` | Optional GPG (AppImage stays unsigned by default) |

## Required secrets

Configure in GitHub repo settings → Secrets → Actions:

| Secret | Used by |
|--------|---------|
| `APPLE_CERTIFICATE` | macOS code signing (base64-encoded `.p12`) |
| `APPLE_CERTIFICATE_PASSWORD` | macOS code signing |
| `APPLE_SIGNING_IDENTITY` | macOS code signing (e.g. `Developer ID Application: Foo (TEAMID)`) |
| `APPLE_ID` | notarytool credentials |
| `APPLE_PASSWORD` | notarytool app-specific password |
| `APPLE_TEAM_ID` | notarytool |
| `WINDOWS_CERTIFICATE` | Authenticode (base64-encoded `.pfx`) |
| `WINDOWS_CERTIFICATE_PASSWORD` | Authenticode |
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri updater signing (separate from code signing) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Tauri updater signing |

Set repo variable `SKIP_NOTARIZE=true` to skip the notarize job for dry runs.

## Tagging a release

```bash
bun scripts/bump-version.ts 0.2.0
git commit -am "chore: release 0.2.0"
git tag -a v0.2.0 -m "release 0.2.0"
git push --tags
```

## Building a release locally

```bash
bun install --cwd frontend
# Updater artifacts (.app.tar.gz/.msi.zip + .sig) need the signing key, so
# they are switched on per build rather than in tauri.conf.json — a plain
# `cargo tauri build` must keep working without the key.
export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.tauri/claude-code-gui.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=...
cargo tauri build --config '{"bundle":{"createUpdaterArtifacts":true}}'
bun scripts/generate-updater-manifest.ts target/release/bundle 0.2.0
```

Generate the key pair once with `cargo tauri signer generate -w ~/.tauri/claude-code-gui.key`
and put the public key in `plugins.updater.pubkey` (it is still the
`REPLACE_WITH_TAURI_SIGNER_PUBKEY` placeholder).

Write the notes from [`release-notes-template.md`](release-notes-template.md);
the manifest's `notes` field is what Settings shows above the install button.

## Updater channels

`plugins.updater.endpoints` may contain `{{channel}}`. The app substitutes
the channel saved in Settings (`stable` or `beta`) before checking
(`updater_check` in `src-tauri/src/commands/updater.rs`), then Tauri fills
in `{{target}}` / `{{current_version}}`. Host one manifest per channel, e.g.
`/stable/darwin-aarch64/0.1.0` and `/beta/darwin-aarch64/0.1.0`.
`updates.example.com` is a placeholder until a real host is chosen.

## Updater manifest format

The updater plugin (`tauri-plugin-updater`) polls the endpoint configured
in `src-tauri/tauri.conf.json` (`plugins.updater.endpoints`). The endpoint
must serve a JSON document of this shape:

```json
{
  "version": "0.2.0",
  "notes": "Release notes…",
  "pub_date": "2026-05-10T12:34:56Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "<output of `tauri signer sign` against the .app.tar.gz>",
      "url": "https://example.com/releases/v0.2.0/Claude%20Code%20GUI_0.2.0_aarch64.app.tar.gz"
    },
    "darwin-x86_64":  { "signature": "...", "url": "..." },
    "linux-x86_64":   { "signature": "...", "url": "..." },
    "windows-x86_64": { "signature": "...", "url": "..." }
  }
}
```

`scripts/generate-updater-manifest.ts` produces a starter manifest from a
directory of artifacts; the `release` job copies it into the GitHub
Release alongside the bundles. Hosting the manifest itself (with a stable
URL that matches `tauri.conf.json`) is the only step the pipeline does
not own — flip `endpoints` to an actual host before shipping 1.0.

## Code signing setup

### macOS

1. Create a Developer ID Application certificate in Apple Developer
   portal.
2. Export as `.p12`, `base64 -i certificate.p12 | pbcopy`, paste into
   `APPLE_CERTIFICATE`.
3. The build runs `cargo tauri build` which picks up the certificate via
   the env vars; `scripts/notarize.sh` then submits each `.dmg` /
   `.app.tar.gz` to notarytool and staples the result.

### Windows

1. Acquire an EV code-signing certificate (DigiCert, Sectigo, etc.).
2. Export `.pfx`, base64-encode, store in `WINDOWS_CERTIFICATE`.
3. `cargo tauri build` invokes `signtool` automatically when the env vars
   are set.

### Linux

AppImage signing is optional and relies on GPG. The pipeline currently
emits an unsigned AppImage; turn on the gpg step in
`.github/workflows/release.yml` once you have a key uploaded.

## Auto-update verification

After a release ships, smoke-test the updater on at least one platform:

1. Install the previous version.
2. Tag the new version, run the release job.
3. Confirm `Settings → Check for updates` shows the new version.
4. Click **Install … and restart** and verify the relaunched binary reports
   the new version.

## Single-instance + deep links

`tauri-plugin-single-instance` is registered in `main.rs`; the second
invocation focuses the running window and re-routes its argv via
`app:single_instance`. The `claude-code-gui://` scheme is registered from
the bundle's Info.plist on macOS and at startup (`register_all`) on Linux
and Windows. A link that launches the app from cold is read with
`getCurrent()` once the frontend is up; a URL arriving on several paths is
handled once. Allowed verbs (locked from
SPEC §8): `install`, `open-agent`. Anything else is logged and dropped
in `frontend/src/lib/deepLink.ts`.

Test deep links manually:

```bash
# macOS
open "claude-code-gui://open-agent/code-reviewer"
open "claude-code-gui://install/marketplace-plugin?source=official"
```
