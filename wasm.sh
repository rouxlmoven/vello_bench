#!/bin/bash
# Build WASM blobs and serve the benchmark UI for browser testing

set -e

SCRIPT_DIR="$(dirname "$0")"

"$SCRIPT_DIR/build.sh"

PORT="${1:-8080}"
echo "Serving benchmark UI at http://localhost:$PORT"
echo "Press Ctrl+C to stop"

cd "$SCRIPT_DIR/ui" && python3 -c "
import http.server, socketserver
socketserver.TCPServer.allow_reuse_address = True
with socketserver.TCPServer(('', $PORT), http.server.SimpleHTTPRequestHandler) as httpd:
    httpd.serve_forever()
"
