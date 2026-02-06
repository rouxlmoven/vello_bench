#!/bin/bash
# Build WASM blobs and run the Tauri app in release mode

set -e

SCRIPT_DIR="$(dirname "$0")"

"$SCRIPT_DIR/build.sh"

cd "$SCRIPT_DIR/vello_bench_tauri"
cargo tauri dev --release
