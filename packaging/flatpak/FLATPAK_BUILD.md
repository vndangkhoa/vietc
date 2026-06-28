# Building the Viet+ Flatpak

## Prerequisites

- Flatpak installed with Flathub remote configured
- `org.gnome.Platform//50` runtime installed
- `org.gnome.Sdk//50` SDK installed
- `org.freedesktop.Sdk.Extension.rust-stable//25.08` installed

### Install dependencies

```bash
flatpak install --user flathub org.gnome.Platform//50
flatpak install --user flathub org.gnome.Sdk//50
flatpak install --user flathub org.freedesktop.Sdk.Extension.rust-stable//25.08
```

---

## Method 1: Quick build script

```bash
cd packaging/flatpak
bash build-flatpak.sh [version]
# e.g. bash build-flatpak.sh 0.1.5
```

Output: `packaging/flatpak/VietPlus-<version>.flatpak`

---

## Method 2: Manual step-by-step

```bash
cd packaging/flatpak

# 1. Clean previous artifacts
rm -rf build-dir repo VietPlus-*.flatpak

# 2. Initialize build directory
#    NOTE: arg order is flatpak build-init DIR APPNAME SDK RUNTIME
flatpak build-init build-dir io.github.vietc.VietPlus \
  org.gnome.Sdk//50 org.gnome.Platform//50

# 3. Copy source code
mkdir -p build-dir/files/src/vietc
rsync -a /path/to/vietc/ build-dir/files/src/vietc/ --exclude=target --exclude=.git

# 4. Build Rust binaries
flatpak build --share=network build-dir sh -c '
  export PATH=/usr/lib/sdk/rust-stable/bin:$PATH
  export CARGO_HOME=/app/cargo
  cd /app/src/vietc
  cargo build --release -p vietc-daemon -p vietc-cli -p vietc-uinputd
'

# 5. Install binaries and icons
flatpak build build-dir sh -c '
  install -Dm755 /app/src/vietc/target/release/vietc /app/bin/vietc-daemon
  install -Dm755 /app/src/vietc/target/release/vietc-cli /app/bin/vietc-cli
  install -Dm755 /app/src/vietc/target/release/vietc-uinputd /app/bin/vietc-uinputd

  install -Dm644 /app/src/vietc/packaging/icons/vietc.svg \
    /app/share/icons/hicolor/scalable/apps/io.github.vietc.VietPlus.svg
  install -Dm644 /app/src/vietc/packaging/icons/vietc-vn.svg \
    /app/share/icons/hicolor/scalable/apps/io.github.vietc.VietPlus.vietc-vn.svg
  install -Dm644 /app/src/vietc/packaging/icons/vietc-en.svg \
    /app/share/icons/hicolor/scalable/apps/io.github.vietc.VietPlus.vietc-en.svg
'

# 6. Finish (set permissions + command)
flatpak build-finish build-dir \
  --socket=x11 \
  --socket=wayland \
  --filesystem=home \
  --share=ipc \
  --talk-name=org.freedesktop.Notifications \
  --talk-name=org.a11y.Bus \
  --command=vietc-daemon

# 7. Export to local repo
flatpak build-export repo build-dir

# 8. Create bundle
flatpak build-bundle repo VietPlus-0.1.5.flatpak io.github.vietc.VietPlus
```

---

## Installation

```bash
# From bundle
flatpak install --user --bundle VietPlus-0.1.5.flatpak

# From local repo
flatpak --user remote-add --no-gpg-verify vietc-repo repo
flatpak --user install vietc-repo io.github.vietc.VietPlus

# Run
flatpak run io.github.vietc.VietPlus
```

---

## Key Notes

- **SDK/RUNTIME order**: `flatpak build-init` takes `SDK` first, then `RUNTIME` (counterintuitive but important — getting this wrong means `/usr/lib/sdk/` won't be mounted)
- **Rust SDK**: must be installed as `org.freedesktop.Sdk.Extension.rust-stable//25.08`; it mounts automatically at `/usr/lib/sdk/rust-stable/`
- **Icons**: all icon files in Flatpak must be prefixed with the app ID (`io.github.vietc.VietPlus.*`) or `flatpak build-export` will skip them
- **Daemon binary name**: Cargo builds the daemon binary as `vietc` (not `vietc-daemon`) in `target/release/`; rename on install to match the desktop file
- **Desktop Categories**: only use registered categories (`Utility`); `InputMethod` is not registered
