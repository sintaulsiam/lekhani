#!/usr/bin/env bash
# Standalone Debian / Ubuntu .deb package generator
# Builds: lekhani-common, ibus-lekhani, fcitx5-lekhani, and lekhani metapackage
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$ROOT_DIR/dist/deb"
VERSION="1.0.0"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64) DEB_ARCH="amd64" ;;
    aarch64) DEB_ARCH="arm64" ;;
    *) DEB_ARCH="$ARCH" ;;
esac

echo "=== Building Lekhani Debian / Ubuntu Packages (v$VERSION, $DEB_ARCH) ==="

mkdir -p "$OUTPUT_DIR"
BUILD_TMP="$ROOT_DIR/target/deb_build"
rm -rf "$BUILD_TMP"
mkdir -p "$BUILD_TMP"

# Ensure release binaries and fcitx5 plugin are built
if [ ! -f "$ROOT_DIR/target/release/lekhani" ] || [ ! -f "$ROOT_DIR/target/release/lekhani-gui" ] || [ ! -f "$ROOT_DIR/target/release/ibus-lekhani" ]; then
    echo "--> Compiling Rust workspace release binaries..."
    cargo build --manifest-path "$ROOT_DIR/Cargo.toml" --release --workspace
fi

if [ -d "/usr/include/Fcitx5" ] || pkg-config --exists fcitx5 2>/dev/null; then
    if [ ! -f "$ROOT_DIR/crates/lekhani-fcitx5/build/fcitx5-lekhani.so" ]; then
        echo "--> Building native Fcitx5 plugin..."
        cmake -B "$ROOT_DIR/crates/lekhani-fcitx5/build" -S "$ROOT_DIR/crates/lekhani-fcitx5" -DCMAKE_BUILD_TYPE=Release
        cmake --build "$ROOT_DIR/crates/lekhani-fcitx5/build" --config Release
    fi
fi

# Helper function to pack a Debian directory into .deb
pack_deb() {
    local PKG_DIR="$1"
    local DEB_NAME="$2"
    local OUT_FILE="$OUTPUT_DIR/$DEB_NAME"

    if command -v dpkg-deb &>/dev/null; then
        dpkg-deb --build --root-owner-group "$PKG_DIR" "$OUT_FILE"
    else
        python3 - <<EOF
import os, sys, tarfile, io, time

pkg_dir = "$PKG_DIR"
out_file = "$OUT_FILE"
os.makedirs(os.path.dirname(out_file), exist_ok=True)

# 1. debian-binary
deb_binary = b"2.0\n"

# 2. control.tar.gz
control_buf = io.BytesIO()
with tarfile.open(fileobj=control_buf, mode="w:gz") as tar:
    control_dir = os.path.join(pkg_dir, "DEBIAN")
    for item in sorted(os.listdir(control_dir)):
        p = os.path.join(control_dir, item)
        ti = tar.gettarinfo(p, arcname=f"./{item}")
        ti.uid = ti.gid = 0
        ti.uname = ti.gname = "root"
        if os.path.isfile(p):
            with open(p, "rb") as f:
                tar.addfile(ti, f)
        else:
            tar.addfile(ti)
control_bytes = control_buf.getvalue()

# 3. data.tar.gz
data_buf = io.BytesIO()
with tarfile.open(fileobj=data_buf, mode="w:gz") as tar:
    for root, dirs, files in os.walk(pkg_dir):
        rel_root = os.path.relpath(root, pkg_dir)
        if rel_root.startswith("DEBIAN"):
            continue
        if rel_root == ".":
            arc_root = "."
        else:
            arc_root = "./" + rel_root
        
        ti = tar.gettarinfo(root, arcname=arc_root)
        ti.uid = ti.gid = 0
        ti.uname = ti.gname = "root"
        tar.addfile(ti)

        for file in sorted(files):
            fpath = os.path.join(root, file)
            frel = os.path.normpath(os.path.join(arc_root, file))
            ti_f = tar.gettarinfo(fpath, arcname=frel)
            ti_f.uid = ti_f.gid = 0
            ti_f.uname = ti_f.gname = "root"
            with open(fpath, "rb") as f:
                tar.addfile(ti_f, f)
data_bytes = data_buf.getvalue()

# Create ar archive
def make_ar_header(name, size):
    name_field = name.ljust(16)
    date_field = str(int(time.time())).ljust(12)
    uid_field = "0".ljust(6)
    gid_field = "0".ljust(6)
    mode_field = "100644".ljust(8)
    size_field = str(size).ljust(10)
    trailer = "\`\n"
    return (name_field + date_field + uid_field + gid_field + mode_field + size_field + trailer).encode('ascii')

with open(out_file, "wb") as f:
    f.write(b"!<arch>\n")
    # debian-binary
    f.write(make_ar_header("debian-binary", len(deb_binary)))
    f.write(deb_binary)
    if len(deb_binary) % 2 != 0:
        f.write(b"\n")
    # control.tar.gz
    f.write(make_ar_header("control.tar.gz", len(control_bytes)))
    f.write(control_bytes)
    if len(control_bytes) % 2 != 0:
        f.write(b"\n")
    # data.tar.gz
    f.write(make_ar_header("data.tar.gz", len(data_bytes)))
    f.write(data_bytes)
    if len(data_bytes) % 2 != 0:
        f.write(b"\n")
EOF
    fi
    echo "  [OK] Created $OUT_FILE"
}

# -------------------------------------------------------------
# 1. lekhani-common
# -------------------------------------------------------------
echo "--> Packaging lekhani-common..."
COMMON_DIR="$BUILD_TMP/lekhani-common"
mkdir -p "$COMMON_DIR/DEBIAN"
mkdir -p "$COMMON_DIR/usr/bin"
mkdir -p "$COMMON_DIR/usr/share/lekhani/layouts"
mkdir -p "$COMMON_DIR/usr/share/lekhani/data"
mkdir -p "$COMMON_DIR/usr/share/lekhani/icons"
mkdir -p "$COMMON_DIR/usr/share/applications"
mkdir -p "$COMMON_DIR/usr/share/metainfo"

install -m755 "$ROOT_DIR/target/release/lekhani-gui" "$COMMON_DIR/usr/bin/"
install -m755 "$ROOT_DIR/target/release/lekhani" "$COMMON_DIR/usr/bin/"

install -m644 "$ROOT_DIR"/data/layouts/*.json "$COMMON_DIR/usr/share/lekhani/layouts/"
install -m644 "$ROOT_DIR"/data/dictionaries/*.json "$COMMON_DIR/usr/share/lekhani/data/"
install -m644 "$ROOT_DIR"/data/dictionaries/*.bin "$COMMON_DIR/usr/share/lekhani/data/" 2>/dev/null || true

install -m644 "$ROOT_DIR/data/icons/128.png" "$COMMON_DIR/usr/share/lekhani/icons/lekhani.png"
if [ -f "$ROOT_DIR/data/icons/lekhani.svg" ]; then
    mkdir -p "$COMMON_DIR/usr/share/icons/hicolor/scalable/apps"
    install -m644 "$ROOT_DIR/data/icons/lekhani.svg" "$COMMON_DIR/usr/share/icons/hicolor/scalable/apps/"
fi
for size in 16 22 24 32 48 64 128 256 512 1024; do
    if [ -f "$ROOT_DIR/data/icons/${size}.png" ]; then
        mkdir -p "$COMMON_DIR/usr/share/icons/hicolor/${size}x${size}/apps"
        install -m644 "$ROOT_DIR/data/icons/${size}.png" "$COMMON_DIR/usr/share/icons/hicolor/${size}x${size}/apps/lekhani.png"
    fi
done

install -m644 "$ROOT_DIR/data/io.github.lekhani.keyboard.desktop" "$COMMON_DIR/usr/share/applications/"
install -m644 "$ROOT_DIR/data/io.github.lekhani.keyboard.metainfo.xml" "$COMMON_DIR/usr/share/metainfo/"

if [ -f "$ROOT_DIR/data/systemd/lekhani-gui.service" ]; then
    mkdir -p "$COMMON_DIR/usr/lib/systemd/user"
    install -m644 "$ROOT_DIR/data/systemd/lekhani-gui.service" "$COMMON_DIR/usr/lib/systemd/user/"
fi

cat <<EOF > "$COMMON_DIR/DEBIAN/control"
Package: lekhani-common
Version: $VERSION-1
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Maintainer: Sintaul Mahdi Siam (Syntenieum) <sintaulsiam@gmail.com>
Depends: hicolor-icon-theme
Recommends: fonts-noto-core | fonts-beng-extra
Description: Modern pure Rust Bengali input method (data, GUI, and CLI)
 Shared layouts, Trie dictionaries, desktop Slint Topbar GUI, and CLI utility for Lekhani.
EOF

cat <<EOF > "$COMMON_DIR/DEBIAN/postinst"
#!/bin/sh
set -e
if which gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
fi
if which update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q || true
fi
exit 0
EOF
chmod 755 "$COMMON_DIR/DEBIAN/postinst"

pack_deb "$COMMON_DIR" "lekhani-common_${VERSION}-1_${DEB_ARCH}.deb"

# -------------------------------------------------------------
# 2. ibus-lekhani
# -------------------------------------------------------------
echo "--> Packaging ibus-lekhani..."
IBUS_DIR="$BUILD_TMP/ibus-lekhani"
mkdir -p "$IBUS_DIR/DEBIAN"
mkdir -p "$IBUS_DIR/usr/bin"
mkdir -p "$IBUS_DIR/usr/share/ibus/component"

install -m755 "$ROOT_DIR/target/release/ibus-lekhani" "$IBUS_DIR/usr/bin/"
install -m644 "$ROOT_DIR/data/ibus/lekhani.xml" "$IBUS_DIR/usr/share/ibus/component/"

if [ -f "$ROOT_DIR/data/systemd/ibus-lekhani.service" ]; then
    mkdir -p "$IBUS_DIR/usr/lib/systemd/user"
    install -m644 "$ROOT_DIR/data/systemd/ibus-lekhani.service" "$IBUS_DIR/usr/lib/systemd/user/"
fi

cat <<EOF > "$IBUS_DIR/DEBIAN/control"
Package: ibus-lekhani
Version: $VERSION-1
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Maintainer: Sintaul Mahdi Siam (Syntenieum) <sintaulsiam@gmail.com>
Depends: lekhani-common (= $VERSION-1), ibus
Description: Lekhani engine for IBus (GNOME / Ubuntu)
 Pure Rust Bengali input method engine integrating with the IBus framework.
EOF

pack_deb "$IBUS_DIR" "ibus-lekhani_${VERSION}-1_${DEB_ARCH}.deb"

# -------------------------------------------------------------
# 3. fcitx5-lekhani
# -------------------------------------------------------------
if [ -f "$ROOT_DIR/crates/lekhani-fcitx5/build/fcitx5-lekhani.so" ]; then
    echo "--> Packaging fcitx5-lekhani..."
    FCITX_DIR="$BUILD_TMP/fcitx5-lekhani"
    mkdir -p "$FCITX_DIR/DEBIAN"
    mkdir -p "$FCITX_DIR/usr/lib/x86_64-linux-gnu/fcitx5"
    mkdir -p "$FCITX_DIR/usr/lib/fcitx5"
    mkdir -p "$FCITX_DIR/usr/share/fcitx5/addon"
    mkdir -p "$FCITX_DIR/usr/share/fcitx5/inputmethod"

    install -m755 "$ROOT_DIR/crates/lekhani-fcitx5/build/fcitx5-lekhani.so" "$FCITX_DIR/usr/lib/x86_64-linux-gnu/fcitx5/"
    install -m755 "$ROOT_DIR/crates/lekhani-fcitx5/build/fcitx5-lekhani.so" "$FCITX_DIR/usr/lib/fcitx5/"
    install -m644 "$ROOT_DIR/data/fcitx5/addon/lekhani.conf" "$FCITX_DIR/usr/share/fcitx5/addon/"
    install -m644 "$ROOT_DIR/data/fcitx5/inputmethod/lekhani.conf" "$FCITX_DIR/usr/share/fcitx5/inputmethod/"

    cat <<EOF > "$FCITX_DIR/DEBIAN/control"
Package: fcitx5-lekhani
Version: $VERSION-1
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Maintainer: Sintaul Mahdi Siam (Syntenieum) <sintaulsiam@gmail.com>
Depends: lekhani-common (= $VERSION-1), fcitx5
Description: Lekhani engine for Fcitx5 (KDE Plasma 6 / Wayland)
 Pure Rust Bengali input method engine integrating with the Fcitx5 framework.
EOF

    pack_deb "$FCITX_DIR" "fcitx5-lekhani_${VERSION}-1_${DEB_ARCH}.deb"
fi

# -------------------------------------------------------------
# 4. lekhani (Metapackage)
# -------------------------------------------------------------
echo "--> Packaging lekhani metapackage..."
META_DIR="$BUILD_TMP/lekhani-meta"
mkdir -p "$META_DIR/DEBIAN"

cat <<EOF > "$META_DIR/DEBIAN/control"
Package: lekhani
Version: $VERSION-1
Section: utils
Priority: optional
Architecture: all
Maintainer: Sintaul Mahdi Siam (Syntenieum) <sintaulsiam@gmail.com>
Depends: lekhani-common (>= $VERSION-1)
Recommends: ibus-lekhani | fcitx5-lekhani
Description: Modern Bengali input method suite (metapackage)
 This metapackage installs the complete Lekhani suite and recommends the appropriate input method engine.
EOF

pack_deb "$META_DIR" "lekhani_${VERSION}-1_all.deb"

echo ""
echo "=========================================================="
echo "  Debian / Ubuntu packages successfully generated in:"
echo "  $OUTPUT_DIR"
echo "=========================================================="
ls -lh "$OUTPUT_DIR"/*.deb
