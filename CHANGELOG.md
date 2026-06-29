# Changelog

## v0.1.5 (2026-06-29)

### Window-Switch Engine Reset
- **Engine state now clears on window switch** — when Alt+Tab'ing between apps, the composition buffer is properly reset before the next keystroke. Previously, keystrokes could still apply Vietnamese tone/mark rules across app boundaries, producing corrupted text.
- **`last_key_time` only on character key presses** — modifier-only events (Alt, Ctrl, Shift) no longer update the gap timer, so the 100 ms inline xprop poll fires reliably after every window switch, regardless of held modifiers.

### Active Window Detection
- **xprop fallback** — `get_active_window_id()` tries `xdotool` first, falls back to `xprop -root _NET_ACTIVE_WINDOW` (preinstalled `x11-utils`). Works under sudo even when xdotool is absent.

### Code Cleanup
- **Removed ~400 lines of dead unsafe code** — entire X11 clipboard shared-state block (unsafe statics, manual Xlib dlopen, SelectionRequest handling) was unused and has been deleted. All related `#[warn(dead_code)]` and `#[warn(static_mut_refs)]` warnings eliminated.
- **Engine dead code removed** — unused methods `is_empty`, `is_tone_or_mark_key`, `process_string`, `last_base_char`, `apply_cluster_mark`, `apply_mark` in `BambooEngine`; `RuleEffect` enum and `special_rules` field in `InputMethodRules`.
- **Production logging** — per-key `eprintln!` removed from evdev loop and uinput paste path. Only startup/error/window-change messages remain (`log_info` to both stderr and file).

### Flatpak Build & System Tray
- **System tray** (`vietc-tray` using ksni/DBus StatusNotifier) is now built and included in the Flatpak bundle. The tray launches the daemon and shows Vietnamese/English mode.
- **Desktop menu entry** — the app now appears when searching **"Viet+"** in the application menu. Search, launch, or uninstall from there.
- **Flatpak command** changed from `vietc-daemon` to `vietc-tray` (the tray spawns the daemon).
- **Tray fixes for Flatpak** — `find_sibling_binary()` now tries `{name}-daemon` fallback; `is_daemon_running()` checks both `vietc` and `vietc-daemon` process names.
- **Fixed `mkdir -p`** — `build-flatpak.sh` now creates `/app/share/applications` before installing the desktop file.

### Active Window Detection (Flatpak fix)
- **Native X11 `_NET_ACTIVE_WINDOW` query** via `dlopen("libX11.so.6")` — added as third fallback in `get_active_window_id()`. Works inside the Flatpak sandbox where `xdotool`/`xprop` are unavailable. No subprocess, no external dependencies.
### Default Mode
- **`start_enabled` now defaults to `true`** — Vietnamese mode is active immediately after launch. Press Ctrl+Space to toggle to English.  
  *(Existing users with a custom config.toml are unaffected — the explicit setting overrides the default.)*

### Tray & Desktop Entry
- **No password prompt inside Flatpak** — `needs_root()` detects Flatpak sandbox (`FLATPAK_ID` or `/app/bin` presence) and skips sudo entirely; the sandbox already has device access via `--device=all`.
- **First-launch flag always written** — the `.first-launch-done` marker is created even when the password prompt is dismissed, preventing repeated prompts.
- **Desktop categories** widened to `Utility;TextTools;X-GNOME-Utilities;` for better visibility in Cinnamon/Mint app menu.
- **Bundle**: `VietPlus-0.1.5.flatpak` (66 MB with tray, runtime `org.gnome.Platform//50`). Warning-free build.

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
