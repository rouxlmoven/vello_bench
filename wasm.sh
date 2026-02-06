#!/bin/bash
# Build WASM blobs and serve the benchmark UI for browser testing

set -e

SCRIPT_DIR="$(dirname "$0")"

"$SCRIPT_DIR/build.sh"

PORT="${1:-8080}"
echo "Serving benchmark UI at http://localhost:$PORT"
echo "Press Ctrl+C to stop"

cd "$SCRIPT_DIR/ui" && python3 -m http.server "$PORT"
