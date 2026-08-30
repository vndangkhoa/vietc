#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Debian Source Package Generator for Launchpad PPA
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    VERSION=$(grep '^version' "$PROJECT_ROOT/engine/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

# Target Ubuntu release: noble (24.04), jammy (22.04), focal (20.04)
SERIES="${2:-noble}"
PPA_REVISION="1~ubuntu${SERIES}1"
PACKAGE_VER="${VERSION}-${PPA_REVISION}"

echo "=== Preparing Launchpad PPA Source Package: vietc ${PACKAGE_VER} (${SERIES}) ==="

BUILD_DIR="/tmp/vietc_ppa_build_${SERIES}"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

# 1. Create clean source tree
echo "1. Preparing source tree with vendored cargo dependencies..."
SOURCE_DIR="$BUILD_DIR/vietc-${VERSION}"
mkdir -p "$SOURCE_DIR"
cp -a "$PROJECT_ROOT"/. "$SOURCE_DIR/"
rm -rf "$SOURCE_DIR"/target "$SOURCE_DIR"/ui/target "$SOURCE_DIR"/vk "$SOURCE_DIR"/.git

# Vendor dependencies for offline Launchpad builders
echo "2. Vendoring cargo crates..."
(cd "$SOURCE_DIR" && cargo vendor --sync ui/Cargo.toml vendor)

# Patch edition = "2024" to "2021" for compatibility with Launchpad Cargo 1.75/1.70
echo "2b. Patching vendored crates for Cargo 1.70/1.75 compatibility..."
for toml in $(grep -rl 'edition = "2024"' "$SOURCE_DIR/vendor/" --include="Cargo.toml" 2>/dev/null); do
    sed -i 's/edition = "2024"/edition = "2021"/g' "$toml"
    # Also patch the .orig copy if it exists
    [ -f "${toml}.orig" ] && sed -i 's/edition = "2024"/edition = "2021"/g' "${toml}.orig"
    # Null out the checksum for this specific crate only
    crate_dir=$(dirname "$toml")
    checksum_file="$crate_dir/.cargo-checksum.json"
    if [ -f "$checksum_file" ]; then
        # Replace the entire "files" object with an empty one to skip verification
        python3 -c "
import json, sys
with open('$checksum_file', 'r') as f:
    data = json.load(f)
data['files'] = {}
with open('$checksum_file', 'w') as f:
    json.dump(data, f)
"
    fi
done

mkdir -p "$SOURCE_DIR/.cargo" "$SOURCE_DIR/ui/.cargo"
cat > "$SOURCE_DIR/.cargo/config.toml" << 'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

cat > "$SOURCE_DIR/ui/.cargo/config.toml" << 'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "../vendor"
EOF

# 2. Package into .orig.tar.gz
echo "3. Creating vietc_${VERSION}.orig.tar.gz..."
(cd "$BUILD_DIR" && tar -czf "vietc_${VERSION}.orig.tar.gz" "vietc-${VERSION}")

# 3. Create debian/ directory
cd "$SOURCE_DIR"
mkdir -p debian/source

# debian/source/format
echo "3.0 (quilt)" > debian/source/format

# debian/control
cat > debian/control << 'EOF'
Source: vietc
Section: utils
Priority: optional
Maintainer: Vo Nguyen Dang Khoa <vonguyendangkhoa@gmail.com>
Build-Depends: debhelper-compat (= 13),
               cargo,
               rustc,
               gcc,
               pkg-config,
               libdbus-1-dev,
               libx11-dev,
               libxtst-dev,
               libxext-dev,
               libevdev-dev,
               libwayland-dev,
               libxkbcommon-dev
Standards-Version: 4.6.2
Homepage: https://github.com/vndangkhoa/vietc

Package: vietc
Architecture: any
Depends: ${shlibs:Depends}, ${misc:Depends}, ibus, dbus
Recommends: libwayland-client0, libx11-6, libxtst6, libdbus-1-3, libxkbcommon0
Description: Vietnamese Input Method for Linux (Wayland & X11)
 Zero-configuration Vietnamese input method supporting Telex and VNI.
 Runs rootless with hybrid backend: native IBus engine on Ubuntu GNOME
 Wayland, zwp_input_method_v2 on Hyprland/Sway, or evdev/uinput on X11.
EOF

# debian/changelog
DATE_R=$(date -R)
cat > debian/changelog << EOF
vietc (${PACKAGE_VER}) ${SERIES}; urgency=medium

  * Release v${VERSION} for Ubuntu ${SERIES} (with vendored dependencies for Launchpad offline builder).

 -- Vo Nguyen Dang Khoa <vonguyendangkhoa@gmail.com>  ${DATE_R}
EOF

# debian/rules
cat > debian/rules << 'EOF'
#!/usr/bin/make -f
%:
	dh $@

override_dh_auto_clean:
	@true

override_dh_auto_build:
	cargo build --offline --release --features "x11,wayland"
	(cd ui && cargo build --offline --release)
	gcc -O2 -o target/release/vietc-xrecord packaging/deb/vietc-xrecord.c -lX11 -lXtst 2>/dev/null || true

override_dh_auto_install:
	install -Dm755 target/release/vietc debian/vietc/usr/bin/vietc-daemon
	install -Dm755 target/release/vietc-cli debian/vietc/usr/bin/vietc-cli
	install -Dm755 target/release/vietc-uinputd debian/vietc/usr/bin/vietc-uinputd
	install -Dm755 target/release/vietcctl debian/vietc/usr/bin/vietcctl
	install -Dm755 ui/target/release/vietc-tray debian/vietc/usr/bin/vietc-tray
	if [ -f target/release/vietc-xrecord ]; then \
		install -Dm755 target/release/vietc-xrecord debian/vietc/usr/bin/vietc-xrecord; \
	fi
	ln -sf vietc-daemon debian/vietc/usr/bin/vietc

	install -Dm644 vietc.toml debian/vietc/etc/vietc/config.toml
	install -Dm644 vietc.service debian/vietc/usr/lib/systemd/user/vietc.service
	install -Dm644 packaging/99-vietc.rules debian/vietc/usr/lib/udev/rules.d/99-vietc.rules
	install -Dm644 packaging/deb/vietc.desktop debian/vietc/usr/share/applications/vietc.desktop
	install -Dm644 packaging/ibus/vietc.xml debian/vietc/usr/share/ibus/component/vietc.xml

	for size in 32x32 48x48 64x64 128x128 256x256; do \
		install -Dm644 packaging/icons/vietc.svg debian/vietc/usr/share/icons/hicolor/$$size/apps/vietc.svg; \
		install -Dm644 packaging/icons/vietc-en.svg debian/vietc/usr/share/icons/hicolor/$$size/apps/vietc-en.svg; \
		install -Dm644 packaging/icons/vietc-vn.svg debian/vietc/usr/share/icons/hicolor/$$size/apps/vietc-vn.svg; \
		install -Dm644 packaging/icons/vietc-tlx.svg debian/vietc/usr/share/icons/hicolor/$$size/apps/vietc-tlx.svg; \
	done
	install -Dm644 packaging/icons/vietc.svg debian/vietc/usr/share/icons/hicolor/scalable/apps/vietc.svg
	install -Dm644 packaging/icons/vietc-en.svg debian/vietc/usr/share/icons/hicolor/scalable/apps/vietc-en.svg
	install -Dm644 packaging/icons/vietc-vn.svg debian/vietc/usr/share/icons/hicolor/scalable/apps/vietc-vn.svg
	install -Dm644 packaging/icons/vietc-tlx.svg debian/vietc/usr/share/icons/hicolor/scalable/apps/vietc-tlx.svg
EOF
chmod +x debian/rules

# debian/postinst, prerm, postrm
cp "$PROJECT_ROOT/packaging/deb/vietc_0.1.9-1_amd64/DEBIAN/postinst" debian/postinst 2>/dev/null || true
cp "$PROJECT_ROOT/packaging/deb/vietc_0.1.9-1_amd64/DEBIAN/prerm" debian/prerm 2>/dev/null || true
cp "$PROJECT_ROOT/packaging/deb/vietc_0.1.9-1_amd64/DEBIAN/postrm" debian/postrm 2>/dev/null || true

# Build source package files
echo "4. Building source package (.dsc, .debian.tar.xz)..."
dpkg-source -b .
dpkg-genchanges -S -sa > "$BUILD_DIR/vietc_${PACKAGE_VER}_source.changes"

echo -e "\n=== Debian source package built successfully! ==="
echo "Changes file: $BUILD_DIR/vietc_${PACKAGE_VER}_source.changes"

# Sign and upload
if [ "${3:-}" = "--upload" ] || [ "${3:-}" = "-u" ]; then
    python3 "$SCRIPT_DIR/upload-ppa.py" "$BUILD_DIR/vietc_${PACKAGE_VER}_source.changes" "khoavo93/vietc"
else
    echo ""
    echo "To sign and upload to Launchpad PPA, run:"
    echo "  python3 $SCRIPT_DIR/upload-ppa.py $BUILD_DIR/vietc_${PACKAGE_VER}_source.changes khoavo93/vietc"
fi
