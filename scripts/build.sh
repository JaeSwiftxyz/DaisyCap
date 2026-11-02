#!/bin/sh
# Build daisycap, assemble dist/DaisyCap.app with its icon, and code-sign it with a
# stable identity so macOS privacy grants (screen, mic, camera, speech)
# survive rebuilds. Usage: scripts/build.sh   (PROFILE=debug for a debug build, UNIVERSAL=1
# for one binary that runs on both chips)
set -eu
cd "$(dirname "$0")/.."
PROFILE="${PROFILE:-release}"
. scripts/env.sh
if [ "${UNIVERSAL:-0}" = 1 ]; then
  # One binary for both chips, built the way scripts/release-macos.sh builds it. whisper-rs-sys is
  # rebuilt every time, since it doesn't notice a change of MACOSX_DEPLOYMENT_TARGET.
  for target in aarch64-apple-darwin x86_64-apple-darwin; do
    cargo clean --release --target "$target" -p whisper-rs-sys
    cargo build --release --locked --target "$target"
  done
  BIN="$CARGO_TARGET_DIR/universal-apple-darwin/release/daisycap"
  mkdir -p "$(dirname "$BIN")"
  lipo -create -output "$BIN" \
    "$CARGO_TARGET_DIR/aarch64-apple-darwin/release/daisycap" \
    "$CARGO_TARGET_DIR/x86_64-apple-darwin/release/daisycap"
else
  if [ "$PROFILE" = release ]; then cargo build --release; else cargo build; fi
  BIN="$CARGO_TARGET_DIR/$PROFILE/daisycap"
fi
APP=dist/DaisyCap.app
STAGE=dist/.DaisyCap.app.new
# Build the bundle beside the old one and swap it in, so a running copy keeps
# its executable until it quits.
rm -rf "$STAGE"
mkdir -p "$STAGE/Contents/MacOS" "$STAGE/Contents/Resources"
cp support/Info.plist "$STAGE/Contents/Info.plist"
cp "$BIN" "$STAGE/Contents/MacOS/DaisyCap"
ICONSET="$CARGO_TARGET_DIR/AppIcon.iconset"
rm -rf "$ICONSET"
"$BIN" icon "$ICONSET" >/dev/null
iconutil -c icns -o "$STAGE/Contents/Resources/AppIcon.icns" "$ICONSET"
IDENTITY="${DAISYCAP_SIGN_IDENTITY:-$(security find-identity -v -p codesigning | awk -F'"' '/Apple Development|Developer ID Application/ {print $2; exit}')}"
if [ -z "$IDENTITY" ]; then
  # Ad-hoc signatures are otherwise known by their hash, which changes on
  # every build, so a granted permission stops matching the new build (it
  # shows as on in System Settings yet DaisyCap keeps asking). Requiring only
  # the identifier keeps grants across rebuilds.
  echo "note: no signing identity found; ad-hoc signing with a stable requirement" >&2
  codesign --force --sign - --identifier com.daisycap.app \
    -r='designated => identifier "com.daisycap.app"' "$STAGE"
else
  codesign --force --sign "$IDENTITY" --identifier com.daisycap.app "$STAGE"
fi
codesign --verify --strict "$STAGE"
rm -rf "$APP.old"
[ -d "$APP" ] && mv "$APP" "$APP.old"
mv "$STAGE" "$APP"
rm -rf "$APP.old"
codesign --verify --strict "$APP"
ln -sf "DaisyCap.app/Contents/MacOS/DaisyCap" dist/daisycap
echo "built $APP (signed: ${IDENTITY:-ad-hoc})"
echo "open:  open $APP      cli: dist/daisycap --help"
