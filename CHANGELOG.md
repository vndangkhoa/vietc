# Changelog

## v0.1.5 (2026-06-28)

### Event Sourcing (privacy-safe architecture)
- **EventStore** replaces `Vec<char>` keystroke history — typed `InputEvent`s (`KeyTyped`, `Backspace`, `Flush`, `Paste`) with `push/pop/clear/raw_keystrokes/pattern_hash`
- **`Engine::replay_events()`** — stateless replay through fresh BambooEngine (replaces `replay_keystrokes()`)
- **`Engine::replay_events_to_commands()`** — computes diff commands (`Type`, `Backspace`) comparing expected vs screen output
- **`EventStore::pattern_hash()`** — sha256 of event type sequence; privacy-safe pattern detection without text recovery
- **Daemon updated** — all `keystroke_history` references migrated to `event_store`; `replay_and_inject()`, `replay_backspace()`, `word_to_commit()`, `replay_reset()` use new Event Sourcing API

### Flatpak Build Fixes
- **Fixed SDK/RUNTIME swap**: `flatpak build-init` arg order is `SDK` then `RUNTIME`; previous `org.gnome.Platform` as SDK meant `/usr/lib/sdk/` was never mounted
- **Rust SDK extension** now auto-mounts at `/usr/lib/sdk/rust-stable/` — no symlinks or file copies needed
- **Icons**: renamed to `io.github.vietc.VietPlus.*` prefix (Flatpak export requires app ID prefix for all icon files)
- **Desktop file**: removed unregistered `InputMethod` category
- **Tray**: `icon_name()` returns Flatpak-prefixed names when running inside Flatpak sandbox (detected via `/app/bin/vietc-daemon`); `icon_pixmap()` programmatic fallback unchanged
- **Bundle**: `VietPlus-0.1.5.flatpak` (46 MB, runtime `org.gnome.Platform//50`)

### Documentation
- `packaging/flatpak/FLATPAK_BUILD.md` — detailed build instructions (prerequisites, manual step-by-step, installation)
- `RELEASE_CHECKLIST.md` — step-by-step release process (bump version, build, test, push, create release)

### Licenses
- MIT license headers (`// SPDX-License-Identifier: MIT`) on all 22 `.rs` files across 6 crates

### Icons
- `packaging/icons/vietc.svg` — app icon (keyboard + VN badge)
- `packaging/icons/vietc-vn.svg` — tray icon (red VN)
- `packaging/icons/vietc-en.svg` — tray icon (gray EN)

---

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
