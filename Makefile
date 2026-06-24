.PHONY: build build-x11 build-wayland build-all build-ui build-tray test test-cli run run-x11 run-wayland clean install install-x11 install-wayland install-ui install-tray install-all-ui install-config appimage deb fmt lint tree

# Build core crates
build:
	cargo build --release

# Build with X11 support
build-x11:
	cargo build --release --features x11

# Build with Wayland IM protocol
build-wayland:
	cargo build --release --features wayland

# Build with all backends
build-all:
	cargo build --release --features "x11,wayland"

# Build settings UI (requires GTK4 + libadwaita)
build-ui:
	cd ui && cargo build --release --bin vietc-settings

# Build tray icon app (requires libdbus-1-dev)
build-tray:
	cd ui && cargo build --release --bin vietc-tray

# Build debug
build-dev:
	cargo build

# Run all tests
test:
	cargo test

# Run the interactive CLI test harness
test-cli:
	cargo run --bin vietc-cli

# Run the daemon (needs root for evdev/uinput)
run: build-dev
	sudo cargo run --bin vietc

# Run daemon with X11 support
run-x11: build-dev
	cargo build --features x11
	sudo cargo run --bin vietc --features x11

# Run daemon with Wayland IM protocol
run-wayland: build-dev
	cargo build --features wayland
	sudo cargo run --bin vietc --features wayland

# Run daemon in release mode
run-release: build
	sudo target/release/vietc

# Install to /usr/local/bin
install: build
	sudo cp target/release/vietc /usr/local/bin/vietc
	@echo "Installed vietc to /usr/local/bin/"

# Install with X11 support
install-x11: build-x11
	sudo cp target/release/vietc /usr/local/bin/vietc
	@echo "Installed vietc (with X11) to /usr/local/bin/"

# Install with Wayland IM protocol
install-wayland: build-wayland
	sudo cp target/release/vietc /usr/local/bin/vietc
	@echo "Installed vietc (with Wayland IM) to /usr/local/bin/"

# Install settings UI
install-ui: build-ui
	sudo cp ui/target/release/vietc-settings /usr/local/bin/vietc-settings
	@echo "Installed vietc-settings to /usr/local/bin/"

# Install tray icon app
install-tray: build-tray
	sudo cp ui/target/release/vietc-tray /usr/local/bin/vietc-tray
	@echo "Installed vietc-tray to /usr/local/bin/"

# Install all UI binaries
install-all-ui: install-ui install-tray

# Install config to user dir
install-config:
	mkdir -p ~/.config/vietc
	cp vietc.toml ~/.config/vietc/config.toml
	@echo "Config installed to ~/.config/vietc/config.toml"

# Build AppImage (requires appimagetool or linuxdeploy)
appimage: build-all
	VERSION=$$(grep '^version' engine/Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/') && \
	bash packaging/appimage/build-appimage.sh "$$VERSION"

# Build .deb package (requires dpkg-deb)
deb: build-all
	VERSION=$$(grep '^version' engine/Cargo.toml | head -1 | sed 's/.*"\(.*\)"/\1/') && \
	bash packaging/deb/build-deb.sh "$$VERSION"

# Clean build artifacts
clean:
	cargo clean
	cd ui && cargo clean
	rm -rf packaging/appimage/AppDir packaging/appimage/*.AppImage packaging/deb/vietc_*

# Format code
fmt:
	cargo fmt
	cd ui && cargo fmt

# Lint
lint:
	cargo clippy -- -D warnings
	cd ui && cargo clippy -- -D warnings

# Show project structure
tree:
	@find . -type f \( -name "*.rs" -o -name "*.toml" \) | grep -v target | sort
