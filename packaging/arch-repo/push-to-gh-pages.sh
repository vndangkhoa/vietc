#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Push Arch Linux Repository to GitHub Pages (gh-pages)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

REMOTE="${1:-github}"
ARCH_DIR="$PROJECT_ROOT/dist/arch/x86_64"

if [ ! -f "$ARCH_DIR/vietc.db" ] || [ ! -f "$ARCH_DIR/vietc.db.tar.gz" ]; then
    echo "⚠️ Arch repo files not found in $ARCH_DIR. Building now..."
    bash "$SCRIPT_DIR/build-repo.sh"
fi

VERSION=$(grep '^version' "$PROJECT_ROOT/engine/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')

echo "=== Pushing Arch Linux Pacman Repository to GitHub Pages (gh-pages) ==="
echo "Remote: $REMOTE"
echo "Version: v$VERSION"

TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# Fetch remote URL
REMOTE_URL=$(git -C "$PROJECT_ROOT" remote get-url "$REMOTE" 2>/dev/null || echo "https://github.com/vndangkhoa/vietc.git")

# Prepare gh-pages branch content
mkdir -p "$TMP_DIR/arch/x86_64"
cp -a "$ARCH_DIR/." "$TMP_DIR/arch/x86_64/"

# Add .nojekyll so GitHub Pages serves raw files properly
touch "$TMP_DIR/.nojekyll"

# Add friendly web landing page
cat > "$TMP_DIR/index.html" << 'EOF'
<!DOCTYPE html>
<html lang="vi">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Viet+ Arch Linux Repository</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; color: #1e293b; background: #f8fafc; line-height: 1.6; }
        .card { background: white; padding: 30px; border-radius: 12px; box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1); }
        h1 { color: #0f172a; margin-top: 0; font-size: 24px; }
        pre { background: #0f172a; color: #f8fafc; padding: 16px; border-radius: 8px; overflow-x: auto; font-size: 14px; }
        code { font-family: "JetBrains Mono", Consolas, Monaco, monospace; }
        .badge { display: inline-block; background: #0284c7; color: white; padding: 4px 10px; border-radius: 20px; font-size: 12px; font-weight: bold; }
    </style>
</head>
<body>
    <div class="card">
        <span class="badge">Arch Linux / CachyOS / Manjaro</span>
        <h1>⚡ Viet+ Official Pacman Repository</h1>
        <p>Kho cài đặt Pacman chính thức giúp bạn cài đặt <strong>Viet+</strong> nhanh chóng bằng lệnh <code>pacman</code> gốc mà không cần biên dịch.</p>

        <h3>1. Thêm kho vào <code>/etc/pacman.conf</code></h3>
        <p>Mở file <code>/etc/pacman.conf</code> bằng quyền root và thêm đoạn sau vào cuối file:</p>
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
git commit -m "Update Arch Linux Pacman repository v${VERSION}"

echo "Pushing to $REMOTE (gh-pages)..."
git push "$REMOTE_URL" gh-pages --force

echo -e "\n✅ Successfully published Pacman repository to GitHub Pages!"
echo "Repository URL: https://vndangkhoa.github.io/vietc/arch/\$arch"
