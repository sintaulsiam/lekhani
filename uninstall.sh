#!/usr/bin/env bash
# Lekhani Local Uninstaller Script
# Runs the uninstallation routine to remove Lekhani binaries, plugins, configs, and caches.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$SCRIPT_DIR/install.sh" --uninstall "$@"
