#!/usr/bin/env bash
# Lekhani GUI Live Hot-Reload Preview Watcher
# Automatically watches crates/lekhani-gui/ui and data/icons/ui
# Recompiles in ~4-5s and refreshes the live window immediately!

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$SCRIPT_DIR"

echo "========================================================"
echo "  🚀 Lekhani GUI Live Hot-Reload Preview Watcher"
echo "  Watching: crates/lekhani-gui/ui & data/icons/ui"
echo "========================================================"

cargo build --package lekhani-gui

pkill -f "target/debug/lekhani-gui" 2>/dev/null || true
./target/debug/lekhani-gui "$@" &
APP_PID=$!

cleanup() {
    echo -e "\n🛑 Stopping live preview..."
    kill $APP_PID 2>/dev/null || true
    pkill -f "target/debug/lekhani-gui" 2>/dev/null || true
    exit 0
}
trap cleanup SIGINT SIGTERM EXIT

while inotifywait -q -e modify,close_write -r crates/lekhani-gui/ui/ data/icons/ui/; do
    echo -e "\n⚡ Detected UI change! Hot-reloading..."
    if cargo build --package lekhani-gui; then
        kill $APP_PID 2>/dev/null || true
        sleep 0.2
        ./target/debug/lekhani-gui "$@" &
        APP_PID=$!
        echo "✨ Live window reloaded!"
    else
        echo "⚠️ Slint syntax error, waiting for next save..."
    fi
done
