# Changelog

## v0.1.7 (2026-07-01)

### Password Auto-Detection

- **AT-SPI2 D-Bus integration**: Queries `org.a11y.atspi.Accessible.GetRole` on the a11y bus (not session bus) to detect password fields. Works in GUI password dialogs and a11y-enabled apps.
- **Process-tree sudo detection**: Scans `pstree` for `sudo`/`passwd` processes — auto-disables Vietnamese when sudo prompts in terminals.
- **Window-title fallback**: Window titles containing "password", "sudo", "mật khẩu" trigger automatic English mode.
- **Window-class fallback**: Known password dialogs (pinentry, polkit, kwallet) detected via `password_apps` config.
- **Periodic re-check**: Re-evaluates password status every 30 keystrokes (catches in-terminal prompts).

### Telex Input Method

- **Telex now fully enabled**: Both VNI and Telex are fully supported. Switch via Ctrl+Shift or tray menu "Input Method > Telex / VNI".
- **Method status file** (`~/.config/vietc/method`): Daemon writes the current method; tray reads it to display.
- **Tray indicator**: Red "VN" for VNI, Blue "TLX" for Telex, Gray "EN" for English mode.
- **Config option**: `toggle_method_key = "shift"` configures the method toggle combo.

### GNOME/Wayland Support

- **GNOME Shell D-Bus integration**: Queries `org.gnome.Shell.Eval` for focused window class, ID, title, and PID — works on Wayland GNOME where xdotool/xprop are unavailable.
- **Window detection chain**: GNOME Shell D-Bus → xprop → wlrctl → xdotool → wmctrl → /proc — works across all environments.
- **Compositor detection**: GNOME/Mutter detected via `pgrep gnome-shell` and `XDG_CURRENT_DESKTOP`.
- **Dependencies**: `dbus` crate (0.9) for AT-SPI2 and GNOME Shell D-Bus.

### Keyboard Grab Safety

- **sigaction without SA_RESTART**: Ctrl+C and SIGTERM now properly interrupt the blocking evdev read, releasing the grab before exit.
- **uinput auto-load**: The injector runs `modprobe uinput` before opening `/dev/uinput`.
- **EINTR handling**: Interrupted system calls are caught and re-check the signal flag.
- **30-second safety timeout**: Auto-releases grab if no events arrive (prevents permanent lockout).

### Clipboard & Injection

- **`wl-copy --paste-once`**: Keeps the clipboard process alive until pasted, eliminating 300-900ms delays on Wayland/GNOME.
- **X11 SelectionRequest log silenced**: No more clipboard spam in the terminal.
- **uinput priority**: uinput is always preferred over X11 XTest injection.

### Config Changes

- **Auto-restore disabled by default**: Prevents space consumption on valid Vietnamese words. Enable via `[auto_restore] enabled = true` if desired.

### CLI Enhancements

- **Pass-through characters**: All characters appear in output (not just engine events).
- **Screen display**: Backspaces properly applied for realistic on-screen view.
- **State reset**: Each input line starts with a clean engine state.
- **New commands**: `:help`, `:status`, `:vi`, `:en`, `:ar on|off`, `:macros`, `:macro add/rm/clear`, `:events`.

### Bug Fixes

- **Double space on Ctrl+Space toggle**: Raw key forwarding now checks engine enabled state.
- **Single-instance lock**: PID written to lock file; stale locks auto-detected and cleaned.
- **xprop/wmctrl fallbacks**: Window detection works without `xdotool` installed.
- **AT-SPI2 a11y bus connection**: Was connecting to session bus; now correctly queries the private a11y bus.
- **Engine state reset between CLI input lines**.

---

## v0.1.6 (2026-06-29)

### uinput-First Injection

- **Injection priority reversed**: uinput (`/dev/uinput`) is now the primary injection backend on X11, with X11 XTest as fallback.
- **X11 XTest keycode fix**: +8 offset applied to all evdev keycodes for XTest compatibility.
- **`paste_via_clipboard()` backspace fixed**: was sending X11 keycode 14 (= "5"), now sends correct keycode 22.

### Window-Switch Detection

- **Active window ID verified on every keystroke**: removed the 100ms guard — catches sub-100ms window switches.

### Input Method

- **Telex disabled in tray**: greyed out as "(next version)". Only VNI was functional.
- **Default input method changed** to `"vni"`.

### Packaging

- **Flatpak and AppImage removed**: only `.deb` packaging is maintained.
- **Postinst improvements**: cleans stale binaries, config files; shows logout popup.

---

## v0.1.5 (2026-06-29)

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
