#!/bin/bash
# Build WASM packages for vello benchmarks

set -e

cd "$(dirname "$0")/.."

echo "Building WASM (scalar)..."
cd vello_bench_wasm
wasm-pack build --target web --out-dir ../vello_bench_tauri/ui/pkg

echo ""
echo "WASM scalar build complete!"
echo "Output: vello_bench_tauri/ui/pkg/"
echo ""

# Optionally build SIMD version
if [ "$1" = "--simd" ]; then
    echo "Building WASM (SIMD128)..."
    RUSTFLAGS='-C target-feature=+simd128' wasm-pack build --target web --out-dir ../vello_bench_tauri/ui/pkg-simd
    echo ""
    echo "WASM SIMD build complete!"
    echo "Output: vello_bench_tauri/ui/pkg-simd/"
fi

echo ""
echo "Note: WASM SIMD is a compile-time feature. To build with SIMD128 support:"
echo "  ./scripts/build-wasm.sh --simd"
echo ""
echo "Or manually:"
echo "  RUSTFLAGS='-C target-feature=+simd128' wasm-pack build --target web --out-dir ../vello_bench_tauri/ui/pkg-simd"
