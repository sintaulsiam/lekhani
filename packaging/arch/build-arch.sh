#!/usr/bin/env bash
# Script to build Arch Linux .pkg.tar.zst packages using makepkg
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$ROOT_DIR/dist/arch"

mkdir -p "$OUTPUT_DIR"

if command -v makepkg &>/dev/null; then
    echo "=== Building Arch Linux packages using makepkg ==="
    # Create source archive from repository root so all crates are included
    VERSION="1.0.0"
    TARBALL="lekhani-$VERSION.tar.gz"
    git -C "$ROOT_DIR" archive --format=tar.gz --prefix="lekhani-$VERSION/" -o "$SCRIPT_DIR/$TARBALL" HEAD
    cd "$SCRIPT_DIR"
    makepkg -f --nodeps --skipinteg
    mv ./*.pkg.tar.zst "$OUTPUT_DIR/" 2>/dev/null || true
    rm -f "$SCRIPT_DIR/$TARBALL"
    echo "Arch packages saved to $OUTPUT_DIR"
else
    echo "Note: 'makepkg' is not installed on this system. You can build on Arch Linux or via GitHub Actions CI."
fi
