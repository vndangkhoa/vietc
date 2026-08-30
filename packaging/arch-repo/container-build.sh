#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Container build runner for Pacman repository
set -euo pipefail

VERSION="${1:-0.1.9}"

echo "1. Installing base-devel & dependencies..."
pacman -Sy --noconfirm base-devel cargo gcc dbus libx11 libxtst wayland libxkbcommon pacman-contrib >/dev/null 2>&1

useradd -m -s /bin/bash builder || true
echo 'builder ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/builder

su builder << BUILDER_EOF
set -euo pipefail
BUILD_TMP=\$(mktemp -d)
cd "\$BUILD_TMP"

cat > PKGBUILD << 'PKGBUILD_EOF'
pkgname=vietc
pkgver=${VERSION}
pkgrel=1
pkgdesc="Vietnamese Input Method for Linux (Wayland & X11) - Zero underline, low latency"
arch=('x86_64')
url="https://github.com/vndangkhoa/vietc"
license=('MIT')
options=('!debug')
depends=('dbus' 'libx11' 'libxtst' 'wayland' 'libxkbcommon')
makedepends=('cargo' 'gcc')
optdepends=(
    'ibus: for IBus engine integration (GNOME Wayland)'
    'wtype: for direct Wayland virtual keyboard support (Hyprland/Sway)'
)
provides=('vietc')
conflicts=('vietc-bin' 'vietc-git')

prepare() {
    cp -a /workspace/. "\$srcdir/"
    rm -rf "\$srcdir/target" "\$srcdir/ui/target" "\$srcdir/vk/target" "\$srcdir/.git" "\$srcdir/dist"
}

build() {
    cd "\$srcdir"
    export RUSTUP_TOOLCHAIN=stable
    cargo build --release --features "x11,wayland"
    (cd ui && cargo build --release)
    gcc -O2 -o target/release/vietc-xrecord packaging/deb/vietc-xrecord.c -lX11 -lXtst 2>/dev/null || true
}

package() {
    cd "\$srcdir"
    install -Dm755 target/release/vietc "\$pkgdir/usr/bin/vietc-daemon"
    install -Dm755 target/release/vietc-cli "\$pkgdir/usr/bin/vietc-cli"
    install -Dm755 target/release/vietc-uinputd "\$pkgdir/usr/bin/vietc-uinputd"
    install -Dm755 target/release/vietcctl "\$pkgdir/usr/bin/vietcctl"
    install -Dm755 ui/target/release/vietc-tray "\$pkgdir/usr/bin/vietc-tray"
    if [ -f target/release/vietc-xrecord ]; then
        install -Dm755 target/release/vietc-xrecord "\$pkgdir/usr/bin/vietc-xrecord"
    fi
    ln -sf vietc-daemon "\$pkgdir/usr/bin/vietc"

    install -Dm644 vietc.toml "\$pkgdir/etc/vietc/config.toml"
    install -Dm644 vietc.service "\$pkgdir/usr/lib/systemd/user/vietc.service"
    install -Dm644 packaging/99-vietc.rules "\$pkgdir/usr/lib/udev/rules.d/99-vietc.rules"
    install -Dm644 packaging/deb/vietc.desktop "\$pkgdir/usr/share/applications/vietc.desktop"
    install -Dm644 packaging/ibus/vietc.xml "\$pkgdir/usr/share/ibus/component/vietc.xml"

    for size in 32x32 48x48 64x64 128x128 256x256; do
        install -Dm644 packaging/icons/vietc.svg "\$pkgdir/usr/share/icons/hicolor/\$size/apps/vietc.svg"
        install -Dm644 packaging/icons/vietc-en.svg "\$pkgdir/usr/share/icons/hicolor/\$size/apps/vietc-en.svg"
        install -Dm644 packaging/icons/vietc-vn.svg "\$pkgdir/usr/share/icons/hicolor/\$size/apps/vietc-vn.svg"
        install -Dm644 packaging/icons/vietc-tlx.svg "\$pkgdir/usr/share/icons/hicolor/\$size/apps/vietc-tlx.svg"
    done
    install -Dm644 packaging/icons/vietc.svg "\$pkgdir/usr/share/icons/hicolor/scalable/apps/vietc.svg"
    install -Dm644 packaging/icons/vietc-en.svg "\$pkgdir/usr/share/icons/hicolor/scalable/apps/vietc-en.svg"
    install -Dm644 packaging/icons/vietc-vn.svg "\$pkgdir/usr/share/icons/hicolor/scalable/apps/vietc-vn.svg"
    install -Dm644 packaging/icons/vietc-tlx.svg "\$pkgdir/usr/share/icons/hicolor/scalable/apps/vietc-tlx.svg"

    install -Dm644 LICENSE "\$pkgdir/usr/share/licenses/\$pkgname/LICENSE"
    install -Dm644 README.md "\$pkgdir/usr/share/doc/\$pkgname/README.md"
}
PKGBUILD_EOF

echo "2. Building package with makepkg..."
makepkg -f --noconfirm

echo "3. Updating repository database with repo-add..."
cp vietc-*.pkg.tar.zst /output/
cd /output
repo-add vietc.db.tar.gz vietc-*.pkg.tar.zst
BUILDER_EOF

chmod -R 777 /output
echo "Arch package and repo database successfully created!"
