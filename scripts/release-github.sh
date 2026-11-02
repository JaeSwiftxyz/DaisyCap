#!/usr/bin/env bash
# Publishes a GitHub release with the universal app as a DMG, a zip and a checksum file.
# The app is ad-hoc signed and not notarized, so macOS asks each person to approve it the first
# time. Writes the artifacts to dist/ and leaves them there.
#
#   scripts/release-github.sh              build, tag and publish the Cargo.toml version
#   scripts/release-github.sh 0.2.5        same, for a version you pass
#   scripts/release-github.sh --local      build the artifacts only, publish nothing
set -euo pipefail
cd "$(dirname "$0")/.."
. scripts/env.sh

local=0
if [ "${1:-}" = "--local" ]; then
    local=1
    shift
fi
version="${1:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
tag="v$version"
repo="JaeSwiftxyz/DaisyCap"

[ -n "$version" ] || { echo "no version in Cargo.toml" >&2; exit 1; }
[ -z "$(git status --porcelain)" ] || { echo "the working tree is not clean" >&2; exit 1; }

if [ "$local" = 0 ]; then
    [ "$(git branch --show-current)" = "main" ] || { echo "switch to main first" >&2; exit 1; }
    command -v gh >/dev/null || { echo "gh is not installed" >&2; exit 1; }
    gh auth status --active >/dev/null 2>&1 || { echo "gh is not logged in" >&2; exit 1; }
    if gh release view "$tag" --repo "$repo" --json tagName >/dev/null 2>&1; then
        echo "release $tag already exists" >&2
        exit 1
    fi
fi

dmg="dist/DaisyCap-$version.dmg"
zip="dist/DaisyCap-$version.zip"
sums="dist/SHA256SUMS-$version.txt"
notes="dist/DaisyCap-$version-notes.md"

scripts/package-dmg.sh
ditto -c -k --sequesterRsrc --keepParent dist/DaisyCap.app "$zip"
( cd dist && shasum -a 256 "DaisyCap-$version.dmg" "DaisyCap-$version.zip" > "SHA256SUMS-$version.txt" )

cat > "$notes" <<EOF
DaisyCap $version for macOS 14 or later, on Apple Silicon and Intel.

Download **DaisyCap-$version.dmg**, or the **.zip** if you prefer a zip.

The app is ad-hoc signed and not notarized, so macOS stops the first launch. Open System Settings, go to Privacy & Security, and choose Open Anyway. After that it opens normally.

This version does not check for updates by itself.

SHA256SUMS-$version.txt has the checksums for both downloads.
EOF

# Check the app as a user gets it, from the finished files.
mount=$(mktemp -d)
trap 'hdiutil detach "$mount" >/dev/null 2>&1 || true; rmdir "$mount" 2>/dev/null || true' EXIT
hdiutil attach -nobrowse -noautoopen -readonly -mountpoint "$mount" "$dmg" >/dev/null
codesign --verify --deep --strict "$mount/DaisyCap.app"
hdiutil detach "$mount" >/dev/null
unzip -tq "$zip" >/dev/null
echo "built $dmg, $zip and $sums"

if [ "$local" = 1 ]; then
    echo "not published (--local)"
    exit 0
fi

gh release create "$tag" "$dmg" "$zip" "$sums" \
    --repo "$repo" --target main --title "DaisyCap $version" --notes-file "$notes"
gh release view "$tag" --repo "$repo" --json url --jq .url
