#!/usr/bin/env bash
# Lekhani Windows Release Compilation & Packaging Pipeline
# Builds 64-bit Windows release binaries (x86_64-pc-windows-gnu) and bundles installer & portable zip
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
TARGET="x86_64-pc-windows-gnu"
VERSION="1.0.0"

echo "=========================================================="
echo "    Lekhani Windows Release Build Pipeline (v$VERSION)    "
echo "    Target: $TARGET                                       "
echo "=========================================================="

mkdir -p "$DIST_DIR"

# 1. Ensure target is installed
if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "Adding rustup target $TARGET..."
    rustup target add "$TARGET"
fi

# 2. Check for MinGW cross-compiler when running on Linux
if [ "$(uname)" = "Linux" ]; then
    if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1 && ! command -v x86_64-w64-mingw32-dlltool >/dev/null 2>&1; then
        echo "  [ERROR] MinGW cross-compilation toolchain not found!"
        echo "  Please install mingw-w64 on your distribution:"
        echo "    - Fedora / RHEL:   sudo dnf install mingw64-gcc mingw64-binutils"
        echo "    - Ubuntu / Debian: sudo apt install mingw-w64"
        echo "    - Arch Linux:      sudo pacman -S mingw-w64-gcc"
        exit 1
    fi
fi

# 3. Build Release Binaries
echo ""
echo "=== Step 1: Compiling Windows Release Binaries ==="
cargo build --manifest-path "$ROOT_DIR/Cargo.toml" \
    --release \
    --target "$TARGET" \
    -p lekhani-cli \
    -p lekhani-gui \
    -p lekhani-ffi

echo "  [OK] Binaries compiled successfully."

# 3. Create Portable Zip Distribution
echo ""
echo "=== Step 2: Creating Portable Windows Zip Distribution ==="
PORTABLE_DIR="$ROOT_DIR/target/portable_windows/lekhani-v$VERSION-windows-x86_64"
ZIP_NAME="lekhani-v$VERSION-windows-x86_64.zip"

rm -rf "$PORTABLE_DIR"
mkdir -p "$PORTABLE_DIR/data"

cp "$ROOT_DIR/target/$TARGET/release/lekhani-gui.exe" "$PORTABLE_DIR/"
cp "$ROOT_DIR/target/$TARGET/release/lekhani.exe" "$PORTABLE_DIR/"
if [ -f "$ROOT_DIR/target/$TARGET/release/lekhani.dll" ]; then
    cp "$ROOT_DIR/target/$TARGET/release/lekhani.dll" "$PORTABLE_DIR/"
fi

cp -r "$ROOT_DIR/data/layouts" "$PORTABLE_DIR/data/"
cp -r "$ROOT_DIR/data/dictionaries" "$PORTABLE_DIR/data/"
cp -r "$ROOT_DIR/data/icons" "$PORTABLE_DIR/data/"
cp "$ROOT_DIR/README.md" "$PORTABLE_DIR/README.txt"

cat << 'EOF' > "$PORTABLE_DIR/START_LEKHANI.bat"
@echo off
start "" "%~dp0lekhani-gui.exe"
EOF

if command -v zip >/dev/null 2>&1; then
    (cd "$ROOT_DIR/target/portable_windows" && zip -r -q "$DIST_DIR/$ZIP_NAME" "lekhani-v$VERSION-windows-x86_64")
    echo "  [OK] Created $DIST_DIR/$ZIP_NAME"
else
    echo "  [INFO] 'zip' command not found. Skipping zip creation."
fi

# 4. Inno Setup Compiler (if available)
echo ""
echo "=== Step 3: Checking Inno Setup Compiler (ISCC) ==="
if command -v iscc >/dev/null 2>&1; then
    echo "Compiling Windows Installer with Inno Setup..."
    iscc "$SCRIPT_DIR/lekhani.iss"
    echo "  [OK] Created $DIST_DIR/Lekhani-v$VERSION-Setup.exe"
else
    echo "  [INFO] 'iscc' not found on this system."
    echo "         To build Lekhani-Setup.exe on Linux, install Inno Setup via wine:"
    echo "         wine iscc packaging/windows/lekhani.iss"
fi

echo ""
echo "=========================================================="
echo "          Windows Build Pipeline Completed!               "
echo "=========================================================="
find "$DIST_DIR" -type f -name "*windows*" -o -name "*Setup*" 2>/dev/null || true
