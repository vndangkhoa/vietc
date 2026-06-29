#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERSION="${1:-0.1.4}"

echo "=== Building Viet+ Flatpak v${VERSION} ==="
cd "$SCRIPT_DIR"

# Clean previous build
rm -rf build-dir repo VietPlus-*.flatpak

# Initialize build directory
# NOTE: arg order is flatpak build-init DIR APPNAME SDK RUNTIME
flatpak build-init build-dir io.github.vietc.VietPlus \
  org.gnome.Sdk//50 org.gnome.Platform//50

# Copy source code
mkdir -p build-dir/files/src/vietc
rsync -a "$PROJECT_ROOT/" build-dir/files/src/vietc/ --exclude=target --exclude=.git

BUILD='export PATH=/usr/lib/sdk/rust-stable/bin:$PATH
export CARGO_HOME=/app/cargo
cd /app/src/vietc'

# Build daemon + CLI + uinputd + tray
echo ""
echo "=== Compiling daemon, CLI, uinputd, tray... ==="
flatpak build --share=network build-dir sh -c "$BUILD && cargo build --release -p vietc-daemon -p vietc-cli -p vietc-uinputd && cargo build --release --manifest-path ui/Cargo.toml"

# Install files
echo ""
echo "=== Installing files... ==="
flatpak build build-dir sh -c "
set -e
install -Dm755 /app/src/vietc/target/release/vietc /app/bin/vietc-daemon
install -Dm755 /app/src/vietc/target/release/vietc-cli /app/bin/vietc-cli
install -Dm755 /app/src/vietc/target/release/vietc-uinputd /app/bin/vietc-uinputd
install -Dm755 /app/src/vietc/ui/target/release/vietc-tray /app/bin/vietc-tray

install -Dm644 /app/src/vietc/packaging/icons/vietc.svg /app/share/icons/hicolor/scalable/apps/io.github.vietc.VietPlus.svg
install -Dm644 /app/src/vietc/packaging/icons/vietc-vn.svg /app/share/icons/hicolor/scalable/apps/io.github.vietc.VietPlus.vietc-vn.svg
                    install -Dm644 /app/src/vietc/packaging/icons/vietc-en.svg /app/share/icons/hicolor/scalable/apps/io.github.vietc.VietPlus.vietc-en.svg

                    mkdir -p /app/share/applications
                    cat > /app/share/applications/io.github.vietc.VietPlus.desktop << END
[Desktop Entry]
Name=Viet+
Comment=Vietnamese Input Method
Exec=/app/bin/vietc-tray
Icon=io.github.vietc.VietPlus
Terminal=false
Type=Application
Categories=Utility;
END

mkdir -p /app/share/metainfo
cat > /app/share/metainfo/io.github.vietc.VietPlus.metainfo.xml << 'XML'
<?xml version='1.0' encoding='utf-8'?>
<component type='desktop-application'>
  <id>io.github.vietc.VietPlus</id>
  <name>Viet+</name>
  <summary>Vietnamese Input Method for Linux</summary>
  <description>
    <p>Zero-configuration Vietnamese input method engine supporting Telex and VNI input methods.</p>
  </description>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <url type='homepage'>https://github.com/vndangkhoa/vietc</url>
  <provides><binary>vietc-daemon</binary></provides>
  <categories><category>Utility</category></categories>
</component>
XML
"

# Finish
echo ""
echo "=== Finalizing build... ==="
flatpak build-finish build-dir \
  --socket=x11 \
  --socket=wayland \
  --socket=session-bus \
  --device=all \
  --share=ipc \
  --talk-name=org.freedesktop.Notifications \
  --talk-name=org.a11y.Bus \
  --talk-name=org.freedesktop.portal.Clipboard \
  --command=vietc-tray

# Export
echo ""
echo "=== Exporting to repository... ==="
flatpak build-export repo build-dir

# Bundle
echo ""
echo "=== Creating bundle... ==="
flatpak build-bundle repo "VietPlus-${VERSION}.flatpak" io.github.vietc.VietPlus

echo ""
echo "=== Done ==="
echo "Package: $SCRIPT_DIR/VietPlus-${VERSION}.flatpak"
echo "Size: $(du -h "$SCRIPT_DIR/VietPlus-${VERSION}.flatpak" | cut -f1)"
echo ""
echo "Install: flatpak install --user --bundle VietPlus-${VERSION}.flatpak"
echo "Run:     flatpak run io.github.vietc.VietPlus"
echo "Search:  'Viet+' in app menu"
