#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Push Arch + Ubuntu repositories to GitHub Pages (gh-pages)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

REMOTE="${1:-github}"
ARCH_DIR="$PROJECT_ROOT/dist/arch/x86_64"
UBUNTU_DIR="$PROJECT_ROOT/dist/ubuntu"

# === Build Arch repo if missing ===
if [ ! -f "$ARCH_DIR/vietc.db" ] || [ ! -f "$ARCH_DIR/vietc.db.tar.gz" ]; then
    echo "⚠️ Arch repo files not found in $ARCH_DIR. Building now..."
    bash "$SCRIPT_DIR/build-repo.sh"
fi

# === Build Ubuntu APT repo if missing ===
if [ ! -f "$UBUNTU_DIR/Packages" ]; then
    echo "⚠️ Ubuntu APT repo not found in $UBUNTU_DIR. Building now..."
    # Find the latest .deb
    DEB_FILE=$(ls -t "$PROJECT_ROOT"/packaging/deb/vietc_*_amd64.deb 2>/dev/null | head -1)
    if [ -z "$DEB_FILE" ]; then
        echo "❌ No .deb file found. Run 'make deb' first."
        exit 1
    fi
    mkdir -p "$UBUNTU_DIR/pool"
    cp "$DEB_FILE" "$UBUNTU_DIR/pool/"

    cd "$UBUNTU_DIR"
    dpkg-scanpackages pool /dev/null > Packages
    gzip -kf Packages

    cat > Release <<RELEASE_EOF
Origin: Viet+ Vietnamese IME
Label: vietc
Suite: stable
Codename: stable
Architectures: amd64
Components: main
Description: Viet+ unofficial APT repository
Date: $(date -Ru)
RELEASE_EOF

    {
      echo "MD5Sum:"
      for f in Packages Packages.gz; do
        echo " $(md5sum "$f" | awk '{print $1}') $(wc -c < "$f") $f"
      done
      echo "SHA256:"
      for f in Packages Packages.gz; do
        echo " $(sha256sum "$f" | awk '{print $1}') $(wc -c < "$f") $f"
      done
    } >> Release

    cd "$PROJECT_ROOT"
fi

VERSION=$(grep '^version' "$PROJECT_ROOT/engine/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')

echo "=== Pushing Arch + Ubuntu repositories to GitHub Pages (gh-pages) ==="
echo "Remote: $REMOTE"
echo "Version: v$VERSION"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Fetch remote URL
REMOTE_URL=$(git -C "$PROJECT_ROOT" remote get-url "$REMOTE" 2>/dev/null || echo "https://github.com/vndangkhoa/vietc.git")

# === Prepare gh-pages content ===

# Arch repo
mkdir -p "$TMP_DIR/arch/x86_64"
cp -a "$ARCH_DIR/." "$TMP_DIR/arch/x86_64/"

# Ubuntu APT repo
mkdir -p "$TMP_DIR/ubuntu/pool"
cp -a "$UBUNTU_DIR/pool/." "$TMP_DIR/ubuntu/pool/"
cp "$UBUNTU_DIR/Packages" "$UBUNTU_DIR/Packages.gz" "$UBUNTU_DIR/Release" "$TMP_DIR/ubuntu/"

# .nojekyll so GitHub Pages serves raw files properly
touch "$TMP_DIR/.nojekyll"

# Landing page with install instructions for both Arch and Ubuntu
cat > "$TMP_DIR/index.html" << 'EOF'
<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Viet+ Linux Repository</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; color: #1e293b; background: #f8fafc; line-height: 1.6; }
        .card { background: white; padding: 30px; border-radius: 12px; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1); margin-bottom: 24px; }
        h1 { color: #0f172a; margin-top: 0; font-size: 24px; }
        h2 { color: #0f172a; font-size: 20px; margin-top: 0; }
        pre { background: #0f172a; color: #f8fafc; padding: 16px; border-radius: 8px; overflow-x: auto; font-size: 14px; }
        code { font-family: "JetBrains Mono", Consolas, Monaco, monospace; }
        .badge { display: inline-block; padding: 4px 10px; border-radius: 20px; font-size: 12px; font-weight: bold; color: white; margin-right: 6px; }
        .badge-arch { background: #1793d1; }
        .badge-ubuntu { background: #e95420; }
        .badge-ppa { background: #77216f; }
        .note { background: #f0f9ff; border-left: 4px solid #0284c7; padding: 12px 16px; border-radius: 0 8px 8px 0; margin: 16px 0; font-size: 14px; }
        a { color: #0284c7; }
    </style>
</head>
<body>
    <div class="card">
        <h1>⚡ Viet+ — Bộ gõ tiếng Việt cho Linux</h1>
        <p>Zero underline, low latency. Hỗ trợ <strong>Wayland</strong> (IBus, zwp_v2) và <strong>X11</strong> (evdev, XTEST).</p>
        <p>GitHub: <a href="https://github.com/vndangkhoa/vietc">github.com/vndangkhoa/vietc</a></p>
    </div>

    <div class="card">
        <span class="badge badge-ubuntu">Ubuntu / Debian</span>
        <h2>Cài đặt qua APT Repository</h2>

        <h3>Cách 1: PPA (Khuyến nghị)</h3>
        <pre><code>sudo add-apt-repository ppa:khoavo93/vietc
sudo apt update
sudo apt install vietc</code></pre>

        <h3>Cách 2: APT repo trên GitHub Pages</h3>
        <p>Thêm repo vào hệ thống:</p>
        <pre><code>echo "deb [trusted=yes] https://vndangkhoa.github.io/vietc/ubuntu/ ./" | sudo tee /etc/apt/sources.list.d/vietc.list
sudo apt update
sudo apt install vietc</code></pre>

        <div class="note">
            💡 <strong>PPA</strong> được khuyến nghị vì Launchpad tự động build cho nhiều phiên bản Ubuntu (Noble 24.04 & Jammy 22.04).
            <br>APT repo trên GitHub Pages chứa bản pre-built cho amd64 — phù hợp nếu bạn không muốn thêm PPA.
        </div>
    </div>

    <div class="card">
        <span class="badge badge-arch">Arch Linux / CachyOS / Manjaro</span>
        <h2>Cài đặt qua Pacman Repository</h2>

        <h3>1. Thêm kho vào <code>/etc/pacman.conf</code></h3>
        <pre><code>[vietc]
SigLevel = Optional TrustAll
Server = https://vndangkhoa.github.io/vietc/arch/$arch</code></pre>

        <h3>2. Cập nhật và Cài đặt</h3>
        <pre><code>sudo pacman -Syu vietc</code></pre>
    </div>
</body>
</html>
EOF

# Initialize clean git repository for gh-pages
cd "$TMP_DIR"
git init -b gh-pages
git config user.name "Vo Nguyen Dang Khoa"
git config user.email "vonguyendangkhoa@gmail.com"
git add -A
git commit -m "Update repos: Arch + Ubuntu v${VERSION}"

echo "Pushing to $REMOTE (gh-pages)..."
git push "$REMOTE_URL" gh-pages --force

echo -e "\n✅ Successfully published repositories to GitHub Pages!"
echo "  Arch: https://vndangkhoa.github.io/vietc/arch/\$arch"
echo "  Ubuntu: https://vndangkhoa.github.io/vietc/ubuntu/"
echo "  Landing: https://vndangkhoa.github.io/vietc/"
