#!/usr/bin/env bash

BASEDIR=$(dirname "$0")
PROJECT_DIR="$(realpath "${BASEDIR}")"

brew install nsis llvm

rustup target add x86_64-pc-windows-msvc

# $(realpath ~/.xwin)
XWIN_DIR=/Users/Shared/xwin

xwin splat --output $XWIN_DIR --disable-symlinks

mkdir -p $PROJECT_DIR/src-tauri/.cargo
echo "
[target.x86_64-pc-windows-msvc]
linker = \"lld\"
rustflags = [
  \"-Lnative=${XWIN_DIR}/crt/lib/x86_64\",
  \"-Lnative=${XWIN_DIR}/sdk/lib/um/x86_64\",
  \"-Lnative=${XWIN_DIR}/sdk/lib/ucrt/x86_64\"
]" >$PROJECT_DIR/src-tauri/.cargo/config.toml

yarn tauri build --target x86_64-pc-windows-msvc
