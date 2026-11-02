#!/usr/bin/env bash
# Packages the universal app as the DMG that jaeswift.xyz serves. The app is ad-hoc signed and not
# notarized, so macOS asks each person to approve it the first time. Updates don't come from a feed:
# "Check for Updates…" opens the DMG link (src/app/updater.rs). Writes dist/DaisyCap-<version>.dmg
# and dist/DaisyCap.dmg, the name the download link serves.
#
#   scripts/package-dmg.sh                build the universal app, then package it
#   SKIP_BUILD=1 scripts/package-dmg.sh   package the app already in dist/
set -euo pipefail
cd "$(dirname "$0")/.."
. scripts/env.sh

if [ "${SKIP_BUILD:-0}" != 1 ]; then
  UNIVERSAL=1 scripts/build.sh
fi
app=dist/DaisyCap.app
[ -d "$app" ] || { echo "no $app: run scripts/build.sh first" >&2; exit 1; }
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)

tmp=$(mktemp -d)
# A failure while an image is mounted would otherwise leave it attached, and rm -rf would delete into it.
cleanup() {
  local mnt
  for mnt in "$tmp/mnt" "$tmp/check"; do
    if [ -d "$mnt" ] && [ "$(stat -f %d "$mnt")" != "$(stat -f %d "$tmp")" ]; then
      hdiutil detach -force "$mnt" >/dev/null || { echo "left an image mounted at $mnt" >&2; return; }
    fi
  done
  rm -rf "$tmp"
}
trap cleanup EXIT

stage=$tmp/stage
mkdir "$stage" "$tmp/mnt" "$tmp/check"
ditto "$app" "$stage/DaisyCap.app"
ln -s /Applications "$stage/Applications"
cp "$app/Contents/Resources/AppIcon.icns" "$stage/.VolumeIcon.icns"
# -srcfolder drops the volume's custom-icon flag, so it is set on a writable image before compression.
hdiutil create -volname DaisyCap -srcfolder "$stage" -format UDRW "$tmp/rw.dmg" >/dev/null
hdiutil attach -nobrowse -noautoopen -mountpoint "$tmp/mnt" "$tmp/rw.dmg" >/dev/null
SetFile -a C "$tmp/mnt"
hdiutil detach "$tmp/mnt" >/dev/null
hdiutil convert "$tmp/rw.dmg" -format UDZO -ov -o "$tmp/DaisyCap.dmg" >/dev/null

# Check the app as a user gets it: dragged out of the finished image.
hdiutil attach -nobrowse -noautoopen -readonly -mountpoint "$tmp/check" "$tmp/DaisyCap.dmg" >/dev/null
ditto "$tmp/check/DaisyCap.app" "$tmp/copied/DaisyCap.app"
hdiutil detach "$tmp/check" >/dev/null
codesign --verify --deep --strict "$tmp/copied/DaisyCap.app"

mv "$tmp/DaisyCap.dmg" "dist/DaisyCap-$version.dmg"
cp "dist/DaisyCap-$version.dmg" dist/DaisyCap.dmg
echo "built dist/DaisyCap-$version.dmg and dist/DaisyCap.dmg"
