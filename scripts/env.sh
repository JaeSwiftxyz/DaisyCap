# Shared build environment for the scripts (sourced, from the repo root).
# Keep build output outside the repo.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/cargo-target/daisycap}"
# Build for the oldest macOS the app supports. Without this, whisper.cpp's C++ targets the SDK's
# version and can call APIs that older systems lack.
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-$(plutil -extract LSMinimumSystemVersion raw support/Info.plist)}"
# whisper.cpp is built with CMake: use the active Xcode's own SDK so its
# linker can read it (a newer Command Line Tools SDK may not be readable).
if [ -z "${SDKROOT:-}" ]; then
  sdk="$(xcode-select -p 2>/dev/null)/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"
  if [ -d "$sdk" ]; then export SDKROOT="$sdk"; fi
fi
