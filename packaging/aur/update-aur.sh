#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — AUR Package Generator & Updater
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    VERSION=$(grep '^version' "$PROJECT_ROOT/engine/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

echo "=== Updating AUR package configurations for Viet+ v${VERSION} ==="

TARBALL_URL="https://github.com/vndangkhoa/vietc/archive/refs/tags/v${VERSION}.tar.gz"
BIN_TARBALL_URL="https://github.com/vndangkhoa/vietc/releases/download/v${VERSION}/vietc_${VERSION}_linux_amd64.tar.gz"

echo "1. Fetching SHA256 for source release (v${VERSION})..."
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

if curl -sLf "$TARBALL_URL" -o "$TEMP_DIR/src.tar.gz" 2>/dev/null; then
    SRC_SHA256=$(sha256sum "$TEMP_DIR/src.tar.gz" | awk '{print $1}')
    echo "   Source SHA256: $SRC_SHA256"
else
    echo "   [Warning] Release tag v${VERSION} not found on GitHub yet. Using SKIP for checksum."
    SRC_SHA256="SKIP"
fi

# Update packaging/aur/PKGBUILD
sed -i "s/^pkgver=.*/pkgver=${VERSION}/" "$SCRIPT_DIR/PKGBUILD"
sed -i "s/^sha256sums=.*/sha256sums=('${SRC_SHA256}')/" "$SCRIPT_DIR/PKGBUILD"

# Generate .SRCINFO if makepkg is available
if command -v makepkg &>/dev/null; then
    echo "2. Generating .SRCINFO for vietc..."
    (cd "$SCRIPT_DIR" && makepkg --printsrcinfo > .SRCINFO)
fi

# Update packaging/aur-bin/PKGBUILD if exists
AUR_BIN_DIR="$PROJECT_ROOT/packaging/aur-bin"
if [ -d "$AUR_BIN_DIR" ]; then
    echo "3. Fetching SHA256 for binary release (v${VERSION})..."
    if curl -sLf "$BIN_TARBALL_URL" -o "$TEMP_DIR/bin.tar.gz" 2>/dev/null; then
        BIN_SHA256=$(sha256sum "$TEMP_DIR/bin.tar.gz" | awk '{print $1}')
        echo "   Binary SHA256: $BIN_SHA256"
    else
        echo "   [Warning] Binary tarball v${VERSION} not found on GitHub Releases yet. Using SKIP for checksum."
        BIN_SHA256="SKIP"
    fi
    sed -i "s/^pkgver=.*/pkgver=${VERSION}/" "$AUR_BIN_DIR/PKGBUILD"
    sed -i "s/^sha256sums=.*/sha256sums=('${BIN_SHA256}')/" "$AUR_BIN_DIR/PKGBUILD"

    if command -v makepkg &>/dev/null; then
        echo "4. Generating .SRCINFO for vietc-bin..."
        (cd "$AUR_BIN_DIR" && makepkg --printsrcinfo > .SRCINFO)
    fi
fi

echo -e "\n=== AUR configuration updated successfully! ==="
echo "To publish to AUR:"
echo "  1. Clone your AUR repo: git clone ssh://aur@aur.archlinux.org/vietc.git /tmp/vietc-aur"
echo "  2. Copy PKGBUILD & .SRCINFO: cp packaging/aur/* /tmp/vietc-aur/"
echo "  3. Commit and push: cd /tmp/vietc-aur && git add . && git commit -m 'Update to v${VERSION}' && git push"
