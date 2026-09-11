#!/usr/bin/env bash
# Script to build Fedora / RHEL RPM packages
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
OUTPUT_DIR="$ROOT_DIR/dist/rpm"
VERSION="3.0.0"

mkdir -p "$OUTPUT_DIR"

if command -v rpmbuild &>/dev/null; then
    echo "=== Building Fedora RPM packages using rpmbuild ==="
    RPMBUILD_DIR="$ROOT_DIR/target/rpmbuild"
    mkdir -p "$RPMBUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

    # Create source tarball
    git archive --format=tar.gz --prefix="lekhani-$VERSION/" -o "$RPMBUILD_DIR/SOURCES/lekhani-$VERSION.tar.gz" HEAD
    cp "$SCRIPT_DIR/lekhani.spec" "$RPMBUILD_DIR/SPECS/"

    rpmbuild --define "_topdir $RPMBUILD_DIR" -ba "$RPMBUILD_DIR/SPECS/lekhani.spec"
    
    find "$RPMBUILD_DIR/RPMS" -name "*.rpm" -exec cp {} "$OUTPUT_DIR/" \;
    find "$RPMBUILD_DIR/SRPMS" -name "*.rpm" -exec cp {} "$OUTPUT_DIR/" \;
    echo "RPM packages saved to $OUTPUT_DIR"
else
    echo "Note: 'rpmbuild' is not installed. To build RPMs locally, install with: sudo dnf install -y rpm-build rpmdevtools"
fi
