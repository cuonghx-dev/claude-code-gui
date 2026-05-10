# Release pipeline

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
3. Confirm the updater plugin shows the new version in
   `Settings → Check for updates`.
4. Apply the update and verify the binary launches.

## Single-instance + deep links

`tauri-plugin-single-instance` is registered in `main.rs`; the second
invocation re-routes its argv to the running window via
`app:single_instance`. `tauri-plugin-deep-link` registers the
`claude-code-gui://` scheme on first launch. Allowed verbs (locked from
SPEC §8): `install`, `open-agent`. Anything else is logged and dropped
in `frontend/src/lib/deepLink.ts`.

Test deep links manually:

```bash
# macOS
open "claude-code-gui://open-agent/code-reviewer"
open "claude-code-gui://install/marketplace-plugin?source=official"
```
