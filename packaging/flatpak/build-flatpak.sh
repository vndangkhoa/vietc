#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERSION="${1:-0.1.4}"

echo "=== Building Viet+ Flatpak v${VERSION} ==="

# Install flatpak-builder if missing
if ! command -v flatpak-builder &>/dev/null; then
    echo "Installing flatpak-builder..."
    sudo apt-get install -y flatpak flatpak-builder
fi

# Add Flathub if missing
if ! flatpak remote-list | grep -q flathub; then
    flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
fi

# Install required runtimes
echo "Installing GNOME Platform & SDK..."
flatpak install -y flathub org.gnome.Platform//47 org.gnome.Sdk//47 2>/dev/null || true
flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//24.08 2>/dev/null || true

# Build
echo "Building Flatpak..."
cd "$SCRIPT_DIR"
flatpak-builder --force-clean --repo=vietc-repo build-dir io.github.vietc.VietPlus.json

# Export to local repo
flatpak build-bundle vietc-repo VietPlus-${VERSION}.flatpak io.github.vietc.VietPlus

echo "=== Done ==="
echo "Package: $SCRIPT_DIR/VietPlus-${VERSION}.flatpak"
echo ""
echo "Install: flatpak install --user VietPlus-${VERSION}.flatpak"
echo "Or via repo: flatpak --user remote-add --no-gpg-verify vietc-local vietc-repo && flatpak run io.github.vietc.VietPlus"
