#!/bin/bash
# Build WASM packages for vello benchmarks (both scalar and SIMD versions).
#
# Uses cargo build + wasm-bindgen directly (instead of wasm-pack) so we can
# build in release mode while preserving debug symbols for Chrome DevTools
# profiling.

set -e

source "$(dirname "$0")/common.sh"
cd "$REPO_ROOT"

# --- Scalar build ---
build_scalar

echo ""

# --- SIMD128 build ---
build_simd

print_build_summary "both"
