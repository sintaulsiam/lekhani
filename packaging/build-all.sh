#!/usr/bin/env bash
# Lekhani Universal Multi-Distro Packaging Pipeline
# Generates packages for Ubuntu/Debian (.deb), Fedora (.rpm), and Arch Linux (.pkg.tar.zst)
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"

VERSION="1.0.0"

echo "=========================================================="
echo "    Lekhani Multi-Distro Package Build Pipeline (v$VERSION)  "
echo "=========================================================="

mkdir -p "$DIST_DIR"

# 1. Compile Release Binaries & Native Fcitx5 Plugin
echo ""
echo "=== Step 1: Compiling Workspace Release Binaries ==="
cargo build --manifest-path "$ROOT_DIR/Cargo.toml" --release --workspace

if [ -d "/usr/include/Fcitx5" ] || pkg-config --exists fcitx5 2>/dev/null; then
    echo "=== Step 2: Compiling Native Fcitx5 Plugin ==="
    cmake -B "$ROOT_DIR/crates/lekhani-fcitx5/build" -S "$ROOT_DIR/crates/lekhani-fcitx5" -DCMAKE_BUILD_TYPE=Release
    cmake --build "$ROOT_DIR/crates/lekhani-fcitx5/build" --config Release
fi

# 2. Build Ubuntu / Debian (.deb) Packages
echo ""
echo "=== Step 3: Building Ubuntu & Debian (.deb) Packages ==="
"$SCRIPT_DIR/debian/build-deb.sh"

# 3. Build Fedora / RHEL (.rpm) Packages
echo ""
echo "=== Step 4: Building Fedora / RHEL (.rpm) Packages ==="
"$SCRIPT_DIR/fedora/build-rpm.sh"

# 4. Build Arch Linux (.pkg.tar.zst) Packages
echo ""
echo "=== Step 5: Building Arch Linux Packages ==="
"$SCRIPT_DIR/arch/build-arch.sh"

# 5. Create Universal Portable Release Archive
echo ""
echo "=== Step 6: Creating Universal Portable Tarball ==="
TARBALL_NAME="lekhani-v$VERSION-linux-$(uname -m).tar.gz"
PORTABLE_DIR="$ROOT_DIR/target/portable_build/lekhani-v$VERSION"
rm -rf "$PORTABLE_DIR"
mkdir -p "$PORTABLE_DIR"

cp "$ROOT_DIR/install.sh" "$PORTABLE_DIR/"
cp "$ROOT_DIR/uninstall.sh" "$PORTABLE_DIR/"
cp -r "$ROOT_DIR/data" "$PORTABLE_DIR/"
mkdir -p "$PORTABLE_DIR/bin"
cp "$ROOT_DIR/target/release/lekhani" "$PORTABLE_DIR/bin/"
cp "$ROOT_DIR/target/release/lekhani-gui" "$PORTABLE_DIR/bin/"
cp "$ROOT_DIR/target/release/ibus-lekhani" "$PORTABLE_DIR/bin/"
if [ -f "$ROOT_DIR/crates/lekhani-fcitx5/build/fcitx5-lekhani.so" ]; then
    mkdir -p "$PORTABLE_DIR/fcitx5"
    cp "$ROOT_DIR/crates/lekhani-fcitx5/build/fcitx5-lekhani.so" "$PORTABLE_DIR/fcitx5/"
fi

tar -czf "$DIST_DIR/$TARBALL_NAME" -C "$ROOT_DIR/target/portable_build" "lekhani-v$VERSION"
echo "  [OK] Created $DIST_DIR/$TARBALL_NAME"

echo ""
echo "=========================================================="
echo "           Packaging Complete! Artifacts in dist/         "
echo "=========================================================="
find "$DIST_DIR" -type f
