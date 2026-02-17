#!/bin/bash
# Shared variables and build functions for the vello benchmark scripts.
#
# Source this file from build.sh, wasm.sh, or tauri.sh:
#   source "$(dirname "$0")/common.sh"

# ---------------------------------------------------------------------------
# Project root (resolved from wherever this file lives)
# ---------------------------------------------------------------------------
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ---------------------------------------------------------------------------
# Build configuration
# ---------------------------------------------------------------------------
WASM_TARGET="wasm32-unknown-unknown"
WASM_PKG="vello_bench_wasm"
RELEASE_DIR="target/${WASM_TARGET}/release"

# Emit full debug info in release builds so Chrome DevTools shows demangled
# Rust function names in performance profiles.
COMMON_RUSTFLAGS="-C debuginfo=2 -C force-frame-pointers=yes -C link-arg=--export-table"

export CARGO_PROFILE_RELEASE_DEBUG=true
export CARGO_PROFILE_RELEASE_STRIP=false

# ---------------------------------------------------------------------------
# Build helpers
# ---------------------------------------------------------------------------

# build_variant LABEL EXTRA_RUSTFLAGS OUT_DIR
#   Runs cargo build + wasm-bindgen for one WASM variant.
#
#   LABEL           Human-readable name shown in log output (e.g. "scalar")
#   EXTRA_RUSTFLAGS Additional RUSTFLAGS appended to the common set (or "")
#   OUT_DIR         wasm-bindgen output directory relative to repo root
build_variant() {
  local label="$1"
  local extra_rustflags="$2"
  local out_dir="$3"

  echo "Building WASM (${label}, release + debug symbols)..."
  RUSTFLAGS="${COMMON_RUSTFLAGS}${extra_rustflags:+ ${extra_rustflags}}" \
    cargo build --target "${WASM_TARGET}" --release -p "${WASM_PKG}"

  echo "Running wasm-bindgen (${label})..."
  wasm-bindgen \
    --target web \
    --out-dir "${out_dir}" \
    --debug \
    --keep-debug \
    "${RELEASE_DIR}/${WASM_PKG}.wasm"
}

build_scalar() {
  build_variant "scalar" "" "ui/pkg"
}

build_simd() {
  build_variant "SIMD128" "-C target-feature=+simd128" "ui/pkg-simd"
}

# Build command strings for use with `cargo watch --shell` and `eval`.
# Generates a single shell command that rebuilds the requested variant(s).
#
# Usage: build_watch_cmd MODE   (MODE = "simd" | "both")
build_watch_cmd() {
  local mode="$1"
  local simd_cmd
  local scalar_cmd

  simd_cmd="RUSTFLAGS='${COMMON_RUSTFLAGS} -C target-feature=+simd128' \
cargo build --target ${WASM_TARGET} --release -p ${WASM_PKG} && \
wasm-bindgen --target web --out-dir ui/pkg-simd --debug --keep-debug \
${RELEASE_DIR}/${WASM_PKG}.wasm"

  scalar_cmd="RUSTFLAGS='${COMMON_RUSTFLAGS}' \
cargo build --target ${WASM_TARGET} --release -p ${WASM_PKG} && \
wasm-bindgen --target web --out-dir ui/pkg --debug --keep-debug \
${RELEASE_DIR}/${WASM_PKG}.wasm"

  if [ "$mode" = "simd" ]; then
    echo "$simd_cmd"
  else
    echo "${scalar_cmd} && ${simd_cmd}"
  fi
}

# ---------------------------------------------------------------------------
# Summary helpers
# ---------------------------------------------------------------------------

print_build_summary() {
  local mode="${1:-both}"

  echo ""
  echo "WASM build complete!"
  if [ "$mode" = "simd" ]; then
    echo "  SIMD: ui/pkg-simd/"
    ls -lh "ui/pkg-simd/${WASM_PKG}_bg.wasm"
  else
    echo "  Scalar: ui/pkg/"
    echo "  SIMD:   ui/pkg-simd/"
    ls -lh "ui/pkg/${WASM_PKG}_bg.wasm" "ui/pkg-simd/${WASM_PKG}_bg.wasm"
  fi
}

# ---------------------------------------------------------------------------
# Cross-Origin Isolation HTTP server
# ---------------------------------------------------------------------------

# serve_with_coi PORT DIRECTORY
#   Starts a Python HTTP server that sets the Cross-Origin-Opener-Policy and
#   Cross-Origin-Embedder-Policy headers required for:
#     - High-resolution performance.now()  (5 μs instead of 100 μs)
#     - SharedArrayBuffer access
#     - Accurate benchmark timing
serve_with_coi() {
  local port="$1"
  local directory="$2"

  python3 -c "
import http.server, socketserver, functools, os

os.chdir('${directory}')

class COIHandler(http.server.SimpleHTTPRequestHandler):
    \"\"\"HTTP handler that adds Cross-Origin Isolation headers.\"\"\"
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cache-Control', 'no-cache')
        super().end_headers()

socketserver.TCPServer.allow_reuse_address = True
with socketserver.TCPServer(('', ${port}), COIHandler) as httpd:
    httpd.serve_forever()
"
}
