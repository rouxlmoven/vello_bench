#!/bin/bash
# Build WASM packages for vello benchmarks (both scalar and SIMD versions)

set -e

cd "$(dirname "$0")/.."

echo "Building WASM (scalar)..."
cd vello_bench_wasm
wasm-pack build --target web --out-dir ../vello_bench_tauri/ui/pkg

echo ""
echo "Building WASM (SIMD128)..."
RUSTFLAGS='-C target-feature=+simd128' wasm-pack build --target web --out-dir ../vello_bench_tauri/ui/pkg-simd

echo ""
echo "WASM builds complete!"
echo "  Scalar: vello_bench_tauri/ui/pkg/"
echo "  SIMD:   vello_bench_tauri/ui/pkg-simd/"
