#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERSION="${1:-0.1.4}"

echo "=== Building Viet+ Flatpak v${VERSION} ==="

# Install required runtimes
flatpak install -y flathub org.gnome.Platform//50 org.gnome.Sdk//50 2>/dev/null || true
flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//25.08 2>/dev/null || true

cd "$SCRIPT_DIR"

# Clean previous build
rm -rf build-dir vietc-repo VietPlus-*.flatpak

# Initialize build directory
flatpak build-init build-dir io.github.vietc.VietPlus \
  org.gnome.Platform//50 org.gnome.Sdk//50

# Add sdk-extensions to metadata
cat > build-dir/metadata << 'EOF'
[Application]
name=io.github.vietc.VietPlus
runtime=org.gnome.Platform/x86_64/50
sdk=org.gnome.Sdk/x86_64/50
sdk-extensions=org.freedesktop.Sdk.Extension.rust-stable
EOF

# Copy source code
mkdir -p build-dir/files/src/vietc
rsync -a "$PROJECT_ROOT/" build-dir/files/src/vietc/ --exclude=target --exclude=.git

# Symlink Rust SDK extension
RUST_FILES=$(find /var/lib/flatpak/runtime/org.freedesktop.Sdk.Extension.rust-stable \
  -name "rustc" -type f 2>/dev/null | head -1 | sed 's|/bin/rustc||')
mkdir -p build-dir/files/usr/lib/sdk
ln -s "$RUST_FILES" build-dir/files/usr/lib/sdk/rust-stable

# Build all Rust binaries inside sandbox
echo "Compiling daemon, CLI, uinputd..."
flatpak build --share=network build-dir sh -c '
  export PATH=/usr/lib/sdk/rust-stable/bin:$PATH
  export CARGO_HOME=/app/cargo
  cd /app/src/vietc
  cargo build --release -p vietc-daemon -p vietc-cli -p vietc-uinputd
'

echo "Compiling system tray..."
flatpak build --share=network build-dir sh -c '
  export PATH=/usr/lib/sdk/rust-stable/bin:$PATH
  export CARGO_HOME=/app/cargo
  cd /app/src/vietc
  cargo build --release --manifest-path ui/Cargo.toml
'

# Install files into sandbox
echo "Installing files..."
flatpak build build-dir sh -c '
  set -e
  install -Dm755 /app/src/vietc/target/release/vietc /app/bin/vietc
  install -Dm755 /app/src/vietc/target/release/vietc-cli /app/bin/vietc-cli
  install -Dm755 /app/src/vietc/target/release/vietc-uinputd /app/bin/vietc-uinputd
  install -Dm755 /app/src/vietc/ui/target/release/vietc-tray /app/bin/vietc-tray
  gcc -O2 -o /app/bin/vietc-xrecord /app/src/vietc/packaging/appimage/vietc-xrecord.c -lX11 -lXtst
  install -Dm755 /app/src/vietc/packaging/flatpak/vietc-wrapper.sh /app/bin/vietc-wrapper.sh
  install -Dm644 /app/src/vietc/packaging/appimage/vietc.desktop \
    /app/share/applications/io.github.vietc.VietPlus.desktop
  sed -i "s/Icon=vietc/Icon=io.github.vietc.VietPlus/g" \
    /app/share/applications/io.github.vietc.VietPlus.desktop
  install -Dm644 /app/src/vietc/vietc.toml /app/etc/vietc/config.toml
  mkdir -p /app/share/icons/hicolor/256x256/apps
  cp /app/src/vietc/packaging/appimage/AppDir/vietc.svg \
    /app/share/icons/hicolor/256x256/apps/io.github.vietc.VietPlus.svg 2>/dev/null || true
  mkdir -p /app/share/metainfo
  cat > /app/share/metainfo/io.github.vietc.VietPlus.metainfo.xml << "XML"
<?xml version="1.0" encoding="utf-8"?>
<component type="desktop-application">
  <id>io.github.vietc.VietPlus</id>
  <name>Viet+</name>
  <summary>Vietnamese Input Method for Linux</summary>
  <description>
    <p>Zero-configuration Vietnamese input method engine supporting Telex and VNI input methods.</p>
  </description>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <url type="homepage">https://github.com/vndangkhoa/vietc</url>
  <provides><binary>vietc</binary></provides>
  <categories><category>Utility</category></categories>
</component>
XML
  mkdir -p /app/share/doc/vietc
  cp /app/src/vietc/README.md /app/share/doc/vietc/ 2>/dev/null || true
  cp /app/src/vietc/LICENSE /app/share/doc/vietc/ 2>/dev/null || true
'

# Finish the build
echo "Finalizing build..."
flatpak build-finish build-dir \
  --socket=x11 \
  --socket=wayland \
  --socket=session-bus \
  --share=ipc \
  --device=all \
  --command=vietc-wrapper.sh

# Export to local repository
echo "Exporting to repository..."
flatpak build-export vietc-repo build-dir

# Create single-file bundle
echo "Creating bundle..."
flatpak build-bundle vietc-repo "VietPlus-${VERSION}.flatpak" io.github.vietc.VietPlus

echo "=== Done ==="
echo "Package: $SCRIPT_DIR/VietPlus-${VERSION}.flatpak ($(du -h "$SCRIPT_DIR/VietPlus-${VERSION}.flatpak" | cut -f1))"
echo ""
echo "Install: flatpak install --user --bundle VietPlus-${VERSION}.flatpak"
echo "Run:     flatpak run io.github.vietc.VietPlus"