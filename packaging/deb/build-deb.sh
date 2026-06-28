#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERSION="${1:-0.1.0}"
PACKAGE="vietc_${VERSION}-1_amd64"
STAGING="$SCRIPT_DIR/$PACKAGE"

echo "=== Building Viet+ .deb package v${VERSION} ==="

# Build binaries (all features: x11 + wayland)
echo "[1/5] Building binaries..."
cargo build --release --features "x11,wayland" --manifest-path "$PROJECT_ROOT/Cargo.toml"
(cd "$PROJECT_ROOT/ui" && export PKG_CONFIG_PATH="/tmp/dbus-dev/extracted/usr/lib/x86_64-linux-gnu/pkgconfig:${PKG_CONFIG_PATH:-}" && export RUSTFLAGS="-L /tmp/dbus-dev/lib" && cargo build --release) || echo "  Warning: UI tray not built (libdbus-1-dev may be missing)"
echo "  Done."

# Clean and create staging
echo "[2/5] Creating staging directory..."
rm -rf "$STAGING"
mkdir -p "$STAGING/DEBIAN"
mkdir -p "$STAGING/usr/bin"
mkdir -p "$STAGING/usr/lib/systemd/user"
mkdir -p "$STAGING/etc/vietc"
mkdir -p "$STAGING/usr/share/applications"
mkdir -p "$STAGING/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$STAGING/usr/share/doc/vietc"
mkdir -p "$STAGING/usr/share/metainfo"

# Copy binaries
echo "[3/5] Installing binaries..."
cp "$PROJECT_ROOT/target/release/vietc" "$STAGING/usr/bin/"
cp "$PROJECT_ROOT/target/release/vietc-cli" "$STAGING/usr/bin/"
# Privileged uinput injection daemon — required for Unicode (Vietnamese) output.
cp "$PROJECT_ROOT/target/release/vietc-uinputd" "$STAGING/usr/bin/"
[ -f "$PROJECT_ROOT/ui/target/release/vietc-tray" ] && cp "$PROJECT_ROOT/ui/target/release/vietc-tray" "$STAGING/usr/bin/"

# Compile and bundle vietc-xrecord (C helper for X11 XRecord keyboard capture)
if command -v gcc &>/dev/null; then
    gcc -O2 -o "$STAGING/usr/bin/vietc-xrecord" "$PROJECT_ROOT/packaging/appimage/vietc-xrecord.c" -lX11 -lXtst \
        && echo "  vietc-xrecord compiled" \
        || echo "  WARNING: vietc-xrecord compile failed (libX11/libXtst dev headers missing)"
else
    echo "  WARNING: no gcc, vietc-xrecord not bundled"
fi

# Desktop file
cp "$PROJECT_ROOT/packaging/appimage/vietc.desktop" "$STAGING/usr/share/applications/"

# Icon (SVG from AppImage build script)
cat > "$STAGING/usr/share/icons/hicolor/256x256/apps/vietc.svg" << 'SVGEOF'
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

# Documentation
cp "$PROJECT_ROOT/README.md" "$STAGING/usr/share/doc/vietc/"
cp "$PROJECT_ROOT/LICENSE" "$STAGING/usr/share/doc/vietc/"

# Config
cp "$PROJECT_ROOT/vietc.toml" "$STAGING/etc/vietc/config.toml"

# Systemd user service
cp "$PROJECT_ROOT/vietc.service" "$STAGING/usr/lib/systemd/user/"

# AppStream metadata
cat > "$STAGING/usr/share/metainfo/io.github.anomalyco.vietc.appdata.xml" << 'XML'
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

# Lintian overrides
mkdir -p "$STAGING/usr/share/lintian/overrides"
cat > "$STAGING/usr/share/lintian/overrides/vietc" << 'LINTIAN'
# Binaries are intentionally placed in /usr/bin without man pages
# as Viet+ is a modern GUI application targeting Wayland/X11.
vietc: no-manual-page *
# Init script helpers are not needed; Viet+ uses systemd --user units.
vietc: no-systemd-service-file-outside-lib
# We bundle the tray alongside the daemon for convenience.
vietc: wrong-section-according-to-package-name
LINTIAN

# Create control file
echo "[4/5] Creating control file..."
cat > "$STAGING/DEBIAN/control" << 'CONTROL'
Package: vietc
Version: VERSION_PLACEHOLDER
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.31), libevdev2 (>= 1.9.0)
Recommends: libwayland-client0 (>= 1.20), libx11-6, libxtst6, xclip
Maintainer: Khoa Vo <vndangkhoa@gmail.com>
Description: Viet+ — Vietnamese Input Method for Linux
 Zero-configuration Vietnamese input method engine supporting
 Telex and VNI input methods. Works natively on both X11 and
 Wayland via evdev uinput injection.
CONTROL
sed -i "s/VERSION_PLACEHOLDER/$VERSION/" "$STAGING/DEBIAN/control"

# Conffiles
echo "/etc/vietc/config.toml" > "$STAGING/DEBIAN/conffiles"

# Maintainer scripts
cat > "$STAGING/DEBIAN/postinst" << 'POSTINST'
#!/bin/sh
set -e
case "$1" in
  configure)
    if command -v systemctl >/dev/null 2>&1; then
      systemctl --system daemon-reload >/dev/null 2>&1 || true
    fi
    ;;
esac
POSTINST
chmod 755 "$STAGING/DEBIAN/postinst"

cat > "$STAGING/DEBIAN/prerm" << 'PRERM'
#!/bin/sh
set -e
case "$1" in
  remove|upgrade|deconfigure)
    if command -v systemctl >/dev/null 2>&1; then
      systemctl --system daemon-reload >/dev/null 2>&1 || true
    fi
    ;;
esac
PRERM
chmod 755 "$STAGING/DEBIAN/prerm"

cat > "$STAGING/DEBIAN/postrm" << 'POSTRM'
#!/bin/sh
set -e
case "$1" in
  purge)
    rm -rf /etc/vietc 2>/dev/null || true
    ;;
esac
POSTRM
chmod 755 "$STAGING/DEBIAN/postrm"

# Build .deb
echo "[5/5] Building .deb package..."
fakeroot dpkg-deb --build "$STAGING" "$SCRIPT_DIR/vietc_${VERSION}-1_amd64.deb"
echo ""
echo "=== Package built: packaging/deb/vietc_${VERSION}-1_amd64.deb ==="
