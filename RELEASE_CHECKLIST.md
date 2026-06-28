# Release Checklist

## When to release

- New feature or bugfix that should be distributed to users
- Flatpak build changes validated
- All tests passing (`cargo test`)

---

## Step-by-step

### 1. Bump version

Update version in:
- `daemon/Cargo.toml`
- `cli/Cargo.toml`
- `engine/Cargo.toml`
- `protocol/Cargo.toml`
- `ui/Cargo.toml`
- `uinputd/Cargo.toml`
- `README.md` version badge

### 2. Update CHANGELOG.md

Add a new entry under the version heading:

```markdown
## vX.Y.Z (YYYY-MM-DD)

### Added
- new features...

### Fixed
- bug fixes...

### Changed
- behavior changes...
```

### 3. Build the Flatpak

```bash
cd packaging/flatpak
bash build-flatpak.sh X.Y.Z
```

Verify the bundle was created:
```bash
ls -lh VietPlus-X.Y.Z.flatpak
```

### 4. Test the Flatpak

```bash
flatpak install --user --bundle VietPlus-X.Y.Z.flatpak
flatpak run io.github.vietc.VietPlus
```

### 5. Commit and push

```bash
git add -A
git commit -m "release: vX.Y.Z — <summary>"
git push origin main
```

### 6. Create a release on Forgejo/GitHub

Attach the Flatpak bundle (`VietPlus-X.Y.Z.flatpak`) as a release asset.

```bash
# Using forgejo-release (if configured)
# Or manually upload via the web UI
```

---

## Quick command

```bash
VERSION=X.Y.Z && \
  sed -i "s/^version = .*/version = \"$VERSION\"/" \
    daemon/Cargo.toml cli/Cargo.toml engine/Cargo.toml \
    protocol/Cargo.toml ui/Cargo.toml uinputd/Cargo.toml && \
  sed -i "s/Version-[0-9.]*-purple/Version-$VERSION-purple/" README.md && \
  echo "Version bumped to $VERSION"
```
