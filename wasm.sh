#!/bin/bash
# Watch-build WASM blobs and serve the benchmark UI for browser testing.
#
# Performs an initial build, then spawns `cargo watch` to automatically rebuild
# on source changes. The dev server sets Cross-Origin Isolation headers so that
# SharedArrayBuffer and high-resolution timers are available.
#
# Usage:
#   ./wasm.sh                     # Watch-build both scalar & SIMD, then serve
#   ./wasm.sh --simd-only         # Watch-build SIMD only, then serve
#   ./wasm.sh -s                  # Same as --simd-only
#   ./wasm.sh --port 9090         # Serve on a custom port
#   ./wasm.sh -s --port 9090      # Combine flags

set -e

source "$(dirname "$0")/common.sh"
cd "$REPO_ROOT"

# ---------------------------------------------------------------------------
# Parse flags
# ---------------------------------------------------------------------------
MODE="both"
PORT=8080

while [[ $# -gt 0 ]]; do
  case "$1" in
    --simd-only|-s)
      MODE="simd"
      shift
      ;;
    --port|-p)
      PORT="$2"
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [--simd-only|-s] [--port|-p PORT]"
      echo ""
      echo "Options:"
      echo "  --simd-only, -s   Only build the SIMD variant (default: both scalar & SIMD)"
      echo "  --port, -p PORT   HTTP server port (default: 8080)"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--simd-only|-s] [--port|-p PORT]"
      exit 1
      ;;
  esac
done

# ---------------------------------------------------------------------------
# Cleanup background processes on exit
# ---------------------------------------------------------------------------
PIDS=()
cleanup() {
  echo ""
  echo "Shutting down..."
  for pid in "${PIDS[@]}"; do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null
}
trap cleanup EXIT INT TERM

# ---------------------------------------------------------------------------
# Initial build
# ---------------------------------------------------------------------------
WATCH_CMD="$(build_watch_cmd "$MODE")"

if [ "$MODE" = "simd" ]; then
  echo "Mode: SIMD only"
else
  echo "Mode: scalar & SIMD"
fi

echo ""
echo "Performing initial build..."
eval "$WATCH_CMD"

print_build_summary "$MODE"

# ---------------------------------------------------------------------------
# Start cargo watch for auto-rebuilding on source changes
# ---------------------------------------------------------------------------
echo ""
echo "Starting file watcher for auto-rebuilding..."
cargo watch \
  --no-vcs-ignores \
  --ignore "ui/pkg" \
  --ignore "ui/pkg-simd" \
  --ignore "ui/*.js" \
  --ignore "ui/*.html" \
  --ignore "ui/*.css" \
  --shell "$WATCH_CMD" &
PIDS+=($!)

# ---------------------------------------------------------------------------
# Start HTTP server with Cross-Origin Isolation headers
# ---------------------------------------------------------------------------
echo ""
echo "Serving benchmark UI at http://localhost:$PORT (cross-origin isolated)"
echo "Press Ctrl+C to stop"
serve_with_coi "$PORT" "$REPO_ROOT/ui" &
PIDS+=($!)

# Keep the script alive until interrupted
wait
