#!/bin/bash
# Build WASM blobs and run the Tauri app in dev mode (release profile).

set -e

source "$(dirname "$0")/common.sh"
cd "$REPO_ROOT"

# Build the WASM packages first
"$REPO_ROOT/build.sh"

# Launch Tauri dev server (watches for Rust source changes automatically)
cd "$REPO_ROOT/vello_bench_tauri"
cargo tauri dev --release --additional-watch-folders ../../vello
