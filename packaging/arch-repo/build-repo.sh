#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
# Viet+ — Custom Pacman Repository Builder
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    VERSION=$(grep '^version' "$PROJECT_ROOT/engine/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')
fi

OUTPUT_DIR="${2:-$PROJECT_ROOT/dist/arch/x86_64}"
mkdir -p "$OUTPUT_DIR"

echo "=== Building Arch Linux Pacman Repository for Viet+ v${VERSION} ==="
echo "Output Directory: $OUTPUT_DIR"

chmod +x "$SCRIPT_DIR/container-build.sh"

# Run inside official Arch Linux container
docker run --rm \
    -v "$PROJECT_ROOT:/workspace:ro" \
    -v "$OUTPUT_DIR:/output" \
    -v "$SCRIPT_DIR/container-build.sh:/container-build.sh:ro" \
    archlinux:latest bash /container-build.sh "$VERSION"

echo -e "\n=== Pacman repository generated successfully! ==="
echo "Files in $OUTPUT_DIR:"
ls -la "$OUTPUT_DIR"
