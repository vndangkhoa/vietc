#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
APPDIR="$SCRIPT_DIR/AppDir"
VERSION="${1:-0.1.0}"

echo "=== Building Viet+ AppImage v${VERSION} ==="

# Clean
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/share/doc/vietc"
mkdir -p "$APPDIR/etc/vietc"

# Build binaries
echo "[1/5] Building binaries..."
cd "$PROJECT_ROOT"
if pkg-config --exists x11 xtst 2>/dev/null; then
    cargo build --release --features "x11,wayland"
    echo "  Built with x11 + wayland"
else
    cargo build --release --features wayland
    echo "  Built with wayland only (X11 libs not found)"
fi

cd "$SCRIPT_DIR"
cd "$PROJECT_ROOT/ui" && cargo build --release 2>/dev/null && cd "$SCRIPT_DIR" || echo "  UI build skipped (missing GTK4 libs)"
cd "$PROJECT_ROOT"

# Copy binaries
echo "[2/5] Installing binaries..."
cp target/release/vietc "$APPDIR/usr/bin/"
cp target/release/vietc-cli "$APPDIR/usr/bin/"
[ -f ui/target/release/vietc-settings ] && cp ui/target/release/vietc-settings "$APPDIR/usr/bin/"
[ -f ui/target/release/vietc-tray ] && cp ui/target/release/vietc-tray "$APPDIR/usr/bin/"

# Desktop integration
echo "[3/5] Installing desktop integration..."
cp "$SCRIPT_DIR/vietc.desktop" "$APPDIR/usr/share/applications/"

# Generate SVG icon
cat > "$APPDIR/usr/share/icons/hicolor/256x256/apps/vietc.svg" << 'SVGEOF'
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

# Convert SVG to PNG if rsvg-convert available
if command -v rsvg-convert &>/dev/null; then
    rsvg-convert -w 256 -h 256 "$APPDIR/usr/share/icons/hicolor/256x256/apps/vietc.svg" \
        -o "$APPDIR/usr/share/icons/hicolor/256x256/apps/vietc.png"
    rm "$APPDIR/usr/share/icons/hicolor/256x256/apps/vietc.svg"
fi

# Copy icon to AppDir root for appimagetool
cp "$APPDIR/usr/share/icons/hicolor/256x256/apps/vietc."{png,svg} "$APPDIR/" 2>/dev/null || true

# Config
echo "[4/5] Installing config..."
cp "$PROJECT_ROOT/vietc.toml" "$APPDIR/etc/vietc/config.toml"
cp "$PROJECT_ROOT/README.md" "$APPDIR/usr/share/doc/vietc/"

# Systemd service
mkdir -p "$APPDIR/usr/lib/systemd/user"
cp "$PROJECT_ROOT/vietc.service" "$APPDIR/usr/lib/systemd/user/"

# Desktop file in AppDir root
cp "$APPDIR/usr/share/applications/vietc.desktop" "$APPDIR/"

echo "[5/5] AppDir ready at: $APPDIR"
echo ""
echo "To build AppImage:"
echo "  appimagetool $APPDIR Viet+-${VERSION}-x86_64.AppImage"
