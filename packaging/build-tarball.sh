#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Generic Linux Tarball Packager
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION=$(grep '^version' "$PROJECT_ROOT/engine/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')

PACKAGE_NAME="vietc_${VERSION}_linux_amd64"
DIST_DIR="$PROJECT_ROOT/target/dist"
STAGING="$DIST_DIR/$PACKAGE_NAME"

echo "=== Building Viet+ release tarball v${VERSION} ==="

# 1. Compile all components
echo "[1/4] Compiling components in release mode..."
cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"
(cd "$PROJECT_ROOT/ui" && cargo build --release)
gcc -O2 -o "$PROJECT_ROOT/target/release/vietc-xrecord" "$PROJECT_ROOT/packaging/deb/vietc-xrecord.c" -lX11 -lXtst

# 2. Recreate staging directory
echo "[2/4] Assembling package structure..."
rm -rf "$DIST_DIR"
mkdir -p "$STAGING/bin"
mkdir -p "$STAGING/udev"
mkdir -p "$STAGING/desktop"
mkdir -p "$STAGING/icons"
mkdir -p "$STAGING/config"

# 3. Copy binaries & rename vietc -> vietc-daemon
cp "$PROJECT_ROOT/target/release/vietc" "$STAGING/bin/vietc-daemon"
cp "$PROJECT_ROOT/target/release/vietc-cli" "$STAGING/bin/"
cp "$PROJECT_ROOT/target/release/vietc-uinputd" "$STAGING/bin/"
cp "$PROJECT_ROOT/ui/target/release/vietc-tray" "$STAGING/bin/"
cp "$PROJECT_ROOT/target/release/vietc-xrecord" "$STAGING/bin/"

# 4. Copy assets & support files
cp "$PROJECT_ROOT/packaging/99-vietc.rules" "$STAGING/udev/"
cp "$PROJECT_ROOT/packaging/deb/vietc.desktop" "$STAGING/desktop/"
cp "$PROJECT_ROOT/vietc.toml" "$STAGING/config/config.toml"
cp "$PROJECT_ROOT/packaging/icons/vietc.svg" "$STAGING/icons/"
cp "$PROJECT_ROOT/packaging/icons/vietc-vn.svg" "$STAGING/icons/"
cp "$PROJECT_ROOT/packaging/icons/vietc-en.svg" "$STAGING/icons/"

cp "$PROJECT_ROOT/install.sh" "$STAGING/"
chmod +x "$STAGING/install.sh"

cp "$PROJECT_ROOT/README.md" "$STAGING/"
cp "$PROJECT_ROOT/LICENSE" "$STAGING/"

# 5. Compress
echo "[3/4] Creating tarball archive..."
(cd "$DIST_DIR" && tar -czf "${PACKAGE_NAME}.tar.gz" "$PACKAGE_NAME")

# 6. Cleanup temp staging
rm -rf "$STAGING"

echo -e "\n=== Package successfully built: ==="
echo "target/dist/${PACKAGE_NAME}.tar.gz"
