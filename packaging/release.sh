#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Master Release & Distribution Automation Script
# Usage: ./packaging/release.sh [VERSION] [--upload-ppa]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

VERSION="${1:-}"
if [ -z "$VERSION" ] || [[ "$VERSION" == --* ]]; then
    VERSION=$(grep '^version' "$PROJECT_ROOT/engine/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

UPLOAD_PPA=false
for arg in "$@"; do
    if [ "$arg" = "--upload-ppa" ]; then
        UPLOAD_PPA=true
    fi
done

echo "========================================================"
echo "  🚀 Viet+ Release Automation System"
echo "  Target Version: v${VERSION}"
echo "========================================================"

# ----------------------------------------------------
# 1. Update Version in Cargo.lock & ui/Cargo.lock to v3
# ----------------------------------------------------
echo -e "\n[1/5] 🔧 Verifying lockfile compatibility (Cargo.lock v3)..."
sed -i 's/^version = 4/version = 3/' "$PROJECT_ROOT/Cargo.lock" "$PROJECT_ROOT/ui/Cargo.lock" 2>/dev/null || true

# ----------------------------------------------------
# 2. Build .deb Package
# ----------------------------------------------------
echo -e "\n[2/5] 📦 Building Debian/Ubuntu .deb package..."
bash "$SCRIPT_DIR/deb/build-deb.sh" "$VERSION"

# ----------------------------------------------------
# 3. Build Generic Linux Tarball
# ----------------------------------------------------
echo -e "\n[3/5] 📦 Building generic Linux release tarball..."
bash "$SCRIPT_DIR/build-tarball.sh"

# ----------------------------------------------------
# 4. Arch Linux Packages (Custom Pacman Repo + AUR)
# ----------------------------------------------------
echo -e "\n[4/5] 🏹 Building Arch Linux Pacman repository & AUR manifests..."
if command -v docker >/dev/null 2>&1; then
    bash "$SCRIPT_DIR/arch-repo/build-repo.sh" "$VERSION"
else
    echo "⚠️ Docker not found; skipping Pacman repo build (will be built by GitHub Actions)."
fi

bash "$SCRIPT_DIR/aur/update-aur.sh" "$VERSION" || true

# ----------------------------------------------------
# 5. Ubuntu Launchpad PPA (Noble 24.04 & Jammy 22.04)
# ----------------------------------------------------
echo -e "\n[5/5] 🟠 Preparing Launchpad PPA source packages..."
PPA_ARGS=()
if [ "$UPLOAD_PPA" = true ]; then
    PPA_ARGS+=("--upload")
fi

echo "  -> Building for Ubuntu 24.04 LTS (noble)..."
bash "$SCRIPT_DIR/ppa/build-source-package.sh" "$VERSION" noble "${PPA_ARGS[@]}"

echo "  -> Building for Ubuntu 22.04 LTS (jammy)..."
bash "$SCRIPT_DIR/ppa/build-source-package.sh" "$VERSION" jammy "${PPA_ARGS[@]}"

# ----------------------------------------------------
# Release Summary
# ----------------------------------------------------
echo -e "\n========================================================"
echo "  ✅ Release v${VERSION} Packaging Completed!"
echo "========================================================"
echo "📁 Generated Artifacts:"
echo "  - Debian .deb:       dist/vietc_${VERSION}_amd64.deb"
echo "  - Tarball:           dist/vietc-v${VERSION}-linux-x86_64.tar.gz"
echo "  - Arch Pacman Repo:  dist/arch/x86_64/ (vietc.db, *.pkg.tar.zst)"
echo "  - AUR Source PKG:    packaging/aur/PKGBUILD"
echo "  - AUR Binary PKG:    packaging/aur-bin/PKGBUILD"
echo "  - Ubuntu PPA (24.04): /tmp/vietc_ppa_build_noble/"
echo "  - Ubuntu PPA (22.04): /tmp/vietc_ppa_build_jammy/"
echo ""
echo "💡 Next Steps:"
echo "  1. Git commit & push:  git add . && git commit -m 'Release v${VERSION}' && git push"
echo "  2. Create Git tag:     git tag v${VERSION} && git push origin v${VERSION}"
echo "     -> GitHub Actions will automatically deploy Pacman repo to GitHub Pages & update Releases!"
echo "========================================================"
