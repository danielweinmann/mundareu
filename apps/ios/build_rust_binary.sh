#!/usr/bin/env bash
set -euo pipefail

export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH"
if ! command -v cargo > /dev/null; then
  echo "error: cargo was not found; install the Rust toolchain from https://rustup.rs and build again" >&2
  exit 1
fi

workspace_root="$(cd "$SRCROOT/../.." && pwd)"
export CARGO_TARGET_DIR="$workspace_root/target/ios"

profile=dev
profile_directory=debug
if [ "$CONFIGURATION" != "Debug" ]; then
  profile=release
  profile_directory=release
fi

executables=()
for architecture in $ARCHS; do
  case "$PLATFORM_NAME/$architecture" in
    iphoneos/arm64) triple=aarch64-apple-ios ;;
    iphonesimulator/arm64) triple=aarch64-apple-ios-sim ;;
    *)
      echo "error: there is no Rust target for $architecture on $PLATFORM_NAME" >&2
      exit 1
      ;;
  esac
  cargo build --profile "$profile" --target "$triple" -p mundareu
  executables+=("$CARGO_TARGET_DIR/$triple/$profile_directory/mundareu")
done

if [ "${#executables[@]}" -eq 1 ]; then
  cp "${executables[0]}" "$TARGET_BUILD_DIR/$EXECUTABLE_PATH"
else
  lipo -create -output "$TARGET_BUILD_DIR/$EXECUTABLE_PATH" "${executables[@]}"
fi
