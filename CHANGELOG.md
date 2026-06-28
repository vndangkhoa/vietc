# Changelog

## v0.1.4 (2026-06-28)

### Flatpak Packaging
- **Flatpak bundle** with all components: daemon, CLI, system tray, uinputd, XRecord, wrapper script
- **System tray icon** via D-Bus StatusNotifierItem (ksni)
- **Build script** `packaging/flatpak/build-flatpak.sh` — automated build from source
- **Permissions:** X11, Wayland, D-Bus session bus, input devices, IPC

### Documentation
- README updated with Flatpak-only install/build instructions

### Clipboard & Injection
- **Fix clipboard-into-text race** — Eliminated race condition where clipboard content leaked into typed text during Unicode injection.
- **CI/CD pipeline** — GitHub Actions workflow for automatic .deb and AppImage builds on push.

### Tests
- **106 tests** passing (72 engine + 16 CLI + 12 protocol + 5 auto-restore + 1 tone placement).

### Releases
- `vietc_0.1.4-1_amd64.deb`, `Viet+-0.1.4-x86_64.AppImage` on GitHub + Forgejo.

---

## v0.1.3 (2026-06-26)

- ua-horn cluster fix, clipboard_context save/restore, control-key consumption
- 106 tests, DEB + AppImage

---

## v0.1.2 (2026-06-26)

- Flush char forwarded as raw key, auto-restore English words
- Tone placement qu/gi/uê/uơ, skip auto-repeat, Enter key

---

## v0.1.1 (2026-06-26)

- Fix Telex tone key consumption, persistent X11 connection

---

## v0.1.0 (2026-06-26)

Initial release — bamboo engine port, evdev capture, uinput injection.
