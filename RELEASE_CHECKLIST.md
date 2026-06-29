# Release Checklist

## When to release

- New feature or bugfix that should be distributed to users
- .deb packaging changes validated
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

### 3. Build the .deb

```bash
make deb
```

Verify the package was created:

```bash
ls -lh packaging/deb/vietc_*.deb
```

### 4. Install & test

```bash
sudo dpkg -i packaging/deb/vietc_X.Y.Z-1_amd64.deb
```

Test:
- Search "Viet+" in the application menu — the tray icon entry should appear
- Launch from menu — tray icon should show, Vietnamese input should work (VNI, Ctrl+Space to toggle)
- The tray should autostart on next login (XDG autostart installed)

### 5. Commit and push

```bash
git add -A
git commit -m "release: vX.Y.Z — <summary>"
git push origin main
```

### 6. Create a release on Forgejo/GitHub

Attach the .deb package (`vietc_X.Y.Z-1_amd64.deb`) as a release asset.

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
