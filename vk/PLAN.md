# Viet+ (vietc) — Session Plan & Resume Point

## Status so far
- **Root cause of "can't type Vietnamese":** IBus-Bamboo's Pre-edit mode flashes a popup that
  Wayland rejects, so the keystroke is committed then rolled back ("stays English"). The real
  direction is that **vietc (your own IME) provides Vietnamese**, independent of IBus/FCitx.
- **Done (user-level, no root):** Built `vietc` (daemon), `vietc-cli`, `vietc-uinputd`,
  `vietc-tray` from `/home/x1/Documents/Projects/vietc`. Stripped the Bamboo GNOME input source
  (`sources = [('xkb','us')]`).
- **Blocked (needs root — `sudo` is unavailable in the agent shell, requires a tty):**
  A setup script is at `/tmp/vietc-setup.sh`. It purges `ibus-unikey`, `ibus-bamboo`, all
  `fcitx5-*`; installs binaries to `/usr/bin`; sets up `/dev/uinput` (udev rule + `input` group +
  `modprobe uinput`); `setcap cap_sys_admin,cap_dac_override` on the daemon; installs
  `vietc.service` + autostart. **You must run:**
  ```
  sudo bash /tmp/vietc-setup.sh
  systemctl --user enable --now vietc.service
  ```
- **Decided:** Build a **standalone virtual-keyboard test tool** (`vietc-vk`) — on-screen
  clickable keyboard + "Run self-test" button — reusing vietc's existing `VirtualKeyboard`
  uinput backend and clipboard verification. Implemented under `vk`.

## vietc-vk (standalone tool) — what it does
- **Stack:** Rust + `evdev` (uinput virtual keyboard, ported from vietc) + `eframe`/`egui` GUI
  (native Wayland; gtk-rs fallback). Clipboard via `wl-paste`/`wl-copy` (Wayland) / `xclip` (X11).
- **Files:** `src/virtual_keyboard.rs`, `src/clipboard.rs`, `src/dictionary.rs`
  (VNI/Telex cases → expected), `src/main.rs` (egui: on-screen keys + Run self-test → PASS/FAIL).
- **How it tests:** the tool creates a uinput virtual keyboard; vietc (already running, grabs all
  keyboards) converts the keystrokes and re-injects via its own ignored uinput device → result
  reaches the focused app / clipboard. Manual = click keys in a focused app; self-test = type the
  dictionary via virtual keyboard, read clipboard, assert. Mirrors vietc's proven
  `daemon_suite.rs` (`tho2i ` → `thời`).
- **Permissions:** `x1` already in `input` group + `99-vietc.rules` set; `setcap
  cap_dac_override+ep` on the `vietc-vk` binary (no `cap_sys_admin` needed — tool never grabs).

## Remaining steps to make it actually run
1. `sudo bash /tmp/vietc-setup.sh`  (sets up vietc + uinput + setcap for vietc daemon)
2. `systemctl --user enable --now vietc.service`  (start vietc tray/daemon)
3. `cd vk && cargo build --release`
4. `sudo setcap cap_dac_override+ep target/release/vietc-vk`
5. `./target/release/vietc-vk`  (with vietc daemon running)

## Caveats to verify
- **Feedback loop:** vietc must grab the test virtual keyboard but ignore its own output device.
  vietc's `device.rs` already does this (the integration test depends on it); confirm with
  `evtest` that the tool's device is grabbed and conversion occurs without duplication.
- **Clipboard dependency:** self-test relies on vietc's clipboard-paste/auto_restore path.
  A clipboard FAIL may mean "verify manually" rather than "conversion broken".
- **Wayland:** egui/winit runs under Wayland; ensure `WAYLAND_DISPLAY` is set. GTK is the fallback.
