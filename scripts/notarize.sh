#!/usr/bin/env bash
# Notarize every .dmg + .app.tar.gz under the given dir using notarytool.
# Requires APPLE_ID / APPLE_PASSWORD (app-specific) / APPLE_TEAM_ID secrets.

set -euo pipefail

bundles_dir="${1:-./bundles}"

if [[ -z "${APPLE_ID:-}" || -z "${APPLE_PASSWORD:-}" || -z "${APPLE_TEAM_ID:-}" ]]; then
  echo "Skipping notarization: APPLE_ID / APPLE_PASSWORD / APPLE_TEAM_ID not set"
  exit 0
fi

xcrun notarytool store-credentials "ccg-notary" \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" \
  --team-id "$APPLE_TEAM_ID" >/dev/null

shopt -s globstar nullglob
for artifact in "$bundles_dir"/**/*.dmg "$bundles_dir"/**/*.app.tar.gz; do
  echo "Submitting $artifact"
  xcrun notarytool submit "$artifact" --keychain-profile "ccg-notary" --wait
  if [[ "$artifact" == *.dmg ]]; then
    xcrun stapler staple "$artifact" || echo "  (staple skipped: not staple-able)"
  fi
done
