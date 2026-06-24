#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERSION="${1:-0.1.0}"
ARCH="amd64"
PKGNAME="vietc"
PKGDIR="$SCRIPT_DIR/${PKGNAME}_${VERSION}_${ARCH}"

echo "=== Building Viet+ .deb v${VERSION} ==="

# Clean
rm -rf "$PKGDIR"
mkdir -p "$PKGDIR/DEBIAN"
chmod 0755 "$PKGDIR/DEBIAN"
mkdir -p "$PKGDIR/usr/bin"
mkdir -p "$PKGDIR/usr/share/applications"
mkdir -p "$PKGDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$PKGDIR/usr/share/doc/vietc"
mkdir -p "$PKGDIR/etc/vietc"
mkdir -p "$PKGDIR/usr/lib/systemd/user"

# Build binaries
echo "[1/6] Building binaries..."
cd "$PROJECT_ROOT"
if pkg-config --exists x11 xtst 2>/dev/null; then
    cargo build --release --features "x11,wayland"
    echo "  Built with x11 + wayland"
else
    cargo build --release --features wayland
    echo "  Built with wayland only (X11 libs not found)"
fi

# Copy binaries
echo "[2/6] Installing binaries..."
cp target/release/vietc "$PKGDIR/usr/bin/"
cp target/release/vietc-cli "$PKGDIR/usr/bin/"

# Try building UI (optional)
cd "$PROJECT_ROOT/ui" && cargo build --release 2>/dev/null && cd "$SCRIPT_DIR" && {
    cp "$PROJECT_ROOT/ui/target/release/vietc-settings" "$PKGDIR/usr/bin/"
    cp "$PROJECT_ROOT/ui/target/release/vietc-tray" "$PKGDIR/usr/bin/"
    echo "  UI binaries included"
} || {
    echo "  UI build skipped (missing GTK4 libs)"
    cd "$SCRIPT_DIR"
}
cd "$PROJECT_ROOT"

# DEBIAN control files
echo "[3/6] Installing control files..."
cp "$SCRIPT_DIR/DEBIAN/control" "$PKGDIR/DEBIAN/control"
sed -i "s/^Version:.*/Version: ${VERSION}/" "$PKGDIR/DEBIAN/control"
cp "$SCRIPT_DIR/DEBIAN/postinst" "$PKGDIR/DEBIAN/"
cp "$SCRIPT_DIR/DEBIAN/prerm" "$PKGDIR/DEBIAN/"
cp "$SCRIPT_DIR/DEBIAN/postrm" "$PKGDIR/DEBIAN/"
chmod 755 "$PKGDIR/DEBIAN/postinst" "$PKGDIR/DEBIAN/prerm" "$PKGDIR/DEBIAN/postrm"

# Desktop integration
echo "[4/6] Installing desktop integration..."
cp "$PROJECT_ROOT/packaging/appimage/vietc.desktop" "$PKGDIR/usr/share/applications/"

# SVG icon
cat > "$PKGDIR/usr/share/icons/hicolor/256x256/apps/vietc.svg" << 'SVGEOF'
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <rect x="20" y="60" width="216" height="140" rx="16" fill="#2d2d2d" stroke="#1a1a1a" stroke-width="4"/>
  <rect x="36" y="76" width="184" height="108" rx="8" fill="#3d3d3d"/>
  <rect x="48" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="78" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="108" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="138" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="168" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="198" y="88" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="54" y="114" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="84" y="114" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="114" y="114" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="144" y="114" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="174" y="114" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="60" y="140" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="90" y="140" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="120" y="140" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="150" y="140" width="24" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="180" y="140" width="42" height="20" rx="3" fill="#f0f0f0"/>
  <rect x="72" y="166" width="112" height="16" rx="3" fill="#f0f0f0"/>
  <circle cx="216" cy="48" r="28" fill="#da251d"/>
  <text x="216" y="56" text-anchor="middle" fill="white" font-size="18" font-weight="bold" font-family="sans-serif">VN</text>
</svg>
SVGEOF

# Convert SVG to PNG if possible
if command -v rsvg-convert &>/dev/null; then
    rsvg-convert -w 256 -h 256 "$PKGDIR/usr/share/icons/hicolor/256x256/apps/vietc.svg" \
        -o "$PKGDIR/usr/share/icons/hicolor/256x256/apps/vietc.png"
fi

# Config and docs
echo "[5/6] Installing config and docs..."
cp "$PROJECT_ROOT/vietc.toml" "$PKGDIR/etc/vietc/config.toml"
cp "$PROJECT_ROOT/README.md" "$PKGDIR/usr/share/doc/vietc/"
cp "$PROJECT_ROOT/LICENSE" "$PKGDIR/usr/share/doc/vietc/"
cp "$PROJECT_ROOT/vietc.service" "$PKGDIR/usr/lib/systemd/user/"

# Calculate installed size
INSTALLED_SIZE=$(du -sk "$PKGDIR" | cut -f1)
sed -i "s/^Installed-Size:.*/Installed-Size: ${INSTALLED_SIZE}/" "$PKGDIR/DEBIAN/control" 2>/dev/null || true

# Fix permissions for dpkg-deb
chmod -R 0755 "$PKGDIR/DEBIAN"
find "$PKGDIR" -type d -exec chmod 0755 {} \;

# Build .deb
echo "[6/6] Building .deb package..."
dpkg-deb --root-owner-group --build "$PKGDIR"

DEBFILE="${PKGNAME}_${VERSION}_${ARCH}.deb"
echo ""
echo "=== Built: $SCRIPT_DIR/$DEBFILE ==="
echo ""
echo "Install with:"
echo "  sudo dpkg -i $DEBFILE"
echo "  sudo apt-get install -f   # fix dependencies if needed"
