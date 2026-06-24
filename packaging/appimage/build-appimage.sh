#!/usr/bin/env bash
set -euo pipefail

# Ensure cargo is in PATH (common for rustup installations)
if ! command -v cargo &>/dev/null; then
    if [ -f "$HOME/.cargo/bin/cargo" ]; then
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
fi

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
cargo build --release
echo "  Built with x11 + wayland"


cd "$SCRIPT_DIR"
cd "$PROJECT_ROOT/ui" && cargo build --release && cd "$SCRIPT_DIR"
cd "$PROJECT_ROOT"

# Copy binaries
echo "[2/5] Installing binaries..."
cp target/release/vietc "$APPDIR/usr/bin/"
cp target/release/vietc-cli "$APPDIR/usr/bin/"
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
else
    # Fallback: generate PNG via Python/Pillow
    python3 -c "
from PIL import Image, ImageDraw
img = Image.new('RGBA', (256, 256), (0,0,0,0))
draw = ImageDraw.Draw(img)
draw.ellipse([(20,20),(236,236)], fill=(218,29,37), outline=(180,20,30), width=4)
try:
    from PIL import ImageFont
    font = ImageFont.truetype('/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf', 80)
except:
    font = ImageFont.load_default()
draw.text((128, 128), 'VN', fill=(255,255,255), font=font, anchor='mm')
img.save('$APPDIR/usr/share/icons/hicolor/256x256/apps/vietc.png')
" 2>/dev/null || echo "  PNG icon generation skipped (no Pillow)"
fi

# Copy icon to AppDir root for appimagetool
cp "$APPDIR/usr/share/icons/hicolor/256x256/apps/vietc."{png,svg} "$APPDIR/" 2>/dev/null || true

# AppStream metadata
mkdir -p "$APPDIR/usr/share/metainfo"
cat > "$APPDIR/usr/share/metainfo/io.github.anomalyco.vietc.appdata.xml" << 'XML'
<?xml version="1.0" encoding="UTF-8"?>
<component type="console-application">
  <id>io.github.anomalyco.vietc</id>
  <name>Viet+</name>
  <summary>Vietnamese Input Method for Linux</summary>
  <description>
    <p>Zero-configuration Vietnamese input method engine supporting Telex and VNI input methods. Works natively on both X11 and Wayland via evdev uinput injection.</p>
  </description>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <url type="homepage">https://github.com/anomalyco/vietc</url>
  <provides><binary>vietc</binary></provides>
  <categories><category>Utility</category></categories>
</component>
XML

# Config
echo "[4/5] Installing config..."
# Use grab=true by default in the AppImage; falls back gracefully for non-root
sed 's/^grab = false/grab = true/' "$PROJECT_ROOT/vietc.toml" > "$APPDIR/etc/vietc/config.toml"
cp "$PROJECT_ROOT/README.md" "$APPDIR/usr/share/doc/vietc/"

# Systemd service
mkdir -p "$APPDIR/usr/lib/systemd/user"
cp "$PROJECT_ROOT/vietc.service" "$APPDIR/usr/lib/systemd/user/"

# Desktop file in AppDir root
cp "$APPDIR/usr/share/applications/vietc.desktop" "$APPDIR/"

# Create custom AppRun script
cat > "$APPDIR/AppRun" << 'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"

# Export our bin dir on PATH so child processes can find sibling binaries
export PATH="$HERE/usr/bin:$PATH"

# Start daemon (kill old non-root one first if we have root)
SUDO_CMD=""

# Fix Wayland env for root: sudo resets XDG_RUNTIME_DIR, breaking wtype/wl-copy.
# Only set WAYLAND_DISPLAY if the user actually has a Wayland session.
if [ "$(id -u)" = "0" ] && [ -z "$XDG_RUNTIME_DIR" ] && [ -n "$SUDO_USER" ]; then
    USER_UID=$(id -u "$SUDO_USER" 2>/dev/null || echo 1000)
    export XDG_RUNTIME_DIR="/run/user/$USER_UID"
    if [ -d "/run/user/$USER_UID" ] && ls "/run/user/$USER_UID/wayland-*" >/dev/null 2>&1; then
        export WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-wayland-0}"
    fi
fi

if command -v pkexec >/dev/null && [ -z "$WAYLAND_DISPLAY" ]; then
    SUDO_CMD="pkexec"
elif [ -n "$WAYLAND_DISPLAY" ]; then
    password=""
    if command -v kdialog >/dev/null; then
        password=$(kdialog --password "Viet+ needs root privileges to grab the keyboard.") || password=""
    elif command -v zenity >/dev/null; then
        password=$(zenity --password --title="Viet+ needs root") || password=""
    elif command -v ssh-askpass >/dev/null; then
        password=$(ssh-askpass "Viet+ needs root privileges") || password=""
    fi
    if [ -n "$password" ]; then
        pkill -x vietc 2>/dev/null; sleep 0.5
        echo "$password" | sudo -S env \
            XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
            WAYLAND_DISPLAY="$WAYLAND_DISPLAY" \
            "$HERE/usr/bin/vietc" >/dev/null &
        DAEMON_PID=$!
    fi
elif command -v sudo >/dev/null; then
    SUDO_CMD="sudo"
fi

if [ -n "$SUDO_CMD" ]; then
    pkill -x vietc 2>/dev/null; sleep 0.5
    if [ "$(id -u)" = "0" ]; then
        # Already root: run daemon with stderr visible (stdout to /dev/null)
        "$HERE/usr/bin/vietc" >/dev/null &
    else
        "$SUDO_CMD" "$HERE/usr/bin/vietc" >/dev/null &
    fi
    DAEMON_PID=$!
fi

if [ -z "$DAEMON_PID" ] && ! pgrep -x vietc >/dev/null; then
    "$HERE/usr/bin/vietc" >/dev/null &
    DAEMON_PID=$!
fi

# Keep the AppImage alive with a tray or settings UI.
# Run as a child (not exec) so daemon cleanup works on exit.
cleanup_daemon() {
    if [ -n "$DAEMON_PID" ]; then
        kill "$DAEMON_PID" 2>/dev/null
        wait "$DAEMON_PID" 2>/dev/null
    fi
}
trap cleanup_daemon EXIT INT TERM

if [ -f "$HERE/usr/bin/vietc-tray" ]; then
    "$HERE/usr/bin/vietc-tray" "$@"
else
    echo "[vietc] Daemon running in foreground. Press Ctrl+C to stop."
    wait "$DAEMON_PID"
fi
EOF
chmod +x "$APPDIR/AppRun"

echo "[5/5] AppDir ready at: $APPDIR"
echo ""

# Auto build if appimagetool exists
if [ -f "$SCRIPT_DIR/appimagetool" ]; then
    echo "=== Running appimagetool FUSE build ==="
    # AppImage inside container/VM sometimes needs --appimage-extract-and-run if FUSE is not mounted
    ARCH=x86_64 "$SCRIPT_DIR/appimagetool" --appimage-extract-and-run "$APPDIR" "$SCRIPT_DIR/Viet+-${VERSION}-x86_64.AppImage"
elif command -v appimagetool &>/dev/null; then
    echo "=== Running system appimagetool ==="
    ARCH=x86_64 appimagetool "$APPDIR" "$SCRIPT_DIR/Viet+-${VERSION}-x86_64.AppImage"
else
    echo "To build AppImage:"
    echo "  appimagetool $APPDIR Viet+-${VERSION}-x86_64.AppImage"
fi
