#!/usr/bin/env bash

set -euo pipefail

NDK_VERSION="25.2.9519653"
# Allow override via env; default to the standard macOS SDK location
NDK_HOME="${ANDROID_NDK_HOME:-$HOME/Library/Android/sdk/ndk/$NDK_VERSION}"

# Detect host platform for the NDK toolchain prebuilt directory
case "$(uname -s)" in
    Darwin) HOST_TAG="darwin-x86_64" ;;
    Linux)  HOST_TAG="linux-x86_64"  ;;
    *)      echo "Unsupported host OS"; exit 1 ;;
esac

TOOLCHAIN="$NDK_HOME/toolchains/llvm/prebuilt/$HOST_TAG/bin"

# Pass linker paths to Cargo via environment variables (avoids hardcoded paths in config.toml)
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$TOOLCHAIN/aarch64-linux-android21-clang"
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="$TOOLCHAIN/armv7a-linux-androideabi21-clang"
export CARGO_TARGET_I686_LINUX_ANDROID_LINKER="$TOOLCHAIN/i686-linux-android21-clang"
export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$TOOLCHAIN/x86_64-linux-android21-clang"

# clear old so files
rm -rf magisk/zygisk/*

echo "==> Building Rust module..."
pushd module/rust

build_target() {
    local abi="$1"
    local target="$2"
    echo "  -> Building $abi ($target)"
    cargo build --release --target "$target"

    mkdir -p ../../magisk/zygisk
    cp "target/$target/release/libhmspush.so" "../../magisk/zygisk/$abi.so"
    echo "  -> Copied $abi.so"
}

build_target "arm64-v8a"   "aarch64-linux-android"
build_target "armeabi-v7a" "armv7-linux-androideabi"
build_target "x86"         "i686-linux-android"
build_target "x86_64"      "x86_64-linux-android"

popd

echo "==> Packaging module zip..."
pushd magisk

version=$(grep '^version=' module.prop | cut -d= -f2)

rm -rf ../build
mkdir -p ../build

zip -r9 "../build/hmspush-zygisk-$version.zip" .

popd

echo "==> Done! Output: build/hmspush-zygisk-$version.zip"
