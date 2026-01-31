#!/bin/bash
# Serve the benchmark UI for browser testing

UI_DIR="$(dirname "$0")/../vello_bench_tauri/ui"
PORT="8080"
BROWSER=""
SERVER_PID=""

# Cleanup function to kill server on exit
cleanup() {
    if [[ -n "$SERVER_PID" ]]; then
        kill $SERVER_PID 2>/dev/null
        wait $SERVER_PID 2>/dev/null
    fi
}

# Set trap to cleanup on Ctrl+C or exit
trap cleanup INT TERM EXIT

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --firefox|-f)
            BROWSER="firefox"
            shift
            ;;
        --safari|-s)
            BROWSER="safari"
            shift
            ;;
        --chrome|-c)
            BROWSER="chrome"
            shift
            ;;
        --port|-p)
            PORT="$2"
            shift 2
            ;;
        *)
            echo "Usage: $0 [--firefox|-f] [--safari|-s] [--chrome|-c] [--port|-p PORT]"
            exit 1
            ;;
    esac
done

URL="http://localhost:$PORT"

# Kill any existing process on the port
EXISTING_PID=$(lsof -ti:$PORT 2>/dev/null)
if [[ -n "$EXISTING_PID" ]]; then
    echo "Killing existing process on port $PORT..."
    kill -9 $EXISTING_PID 2>/dev/null
    sleep 0.5
fi

# Find python
if command -v python3 &> /dev/null; then
    PYTHON="python3"
elif command -v python &> /dev/null; then
    PYTHON="python"
else
    echo "Error: Python not found. Please install Python or use another HTTP server."
    exit 1
fi

echo "Serving benchmark UI at $URL"
echo "Press Ctrl+C to stop"
echo ""

# Start server in background
cd "$UI_DIR" && $PYTHON -m http.server "$PORT" &
SERVER_PID=$!

# Wait for server to start
sleep 0.5

# Open browser if specified
if [[ -n "$BROWSER" ]]; then
    echo "Opening $BROWSER..."
    case "$BROWSER" in
        firefox)
            open -a "Firefox" "$URL" 2>/dev/null || firefox "$URL" &
            ;;
        safari)
            open -a "Safari" "$URL" &
            ;;
        chrome)
            open -a "Google Chrome" "$URL" 2>/dev/null || google-chrome "$URL" &
            ;;
    esac
fi

# Wait for server process (Ctrl+C will trigger cleanup)
wait $SERVER_PID
