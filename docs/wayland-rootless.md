# vietc on Wayland — Rootless Operation

This document records how vietc runs as a normal user (no root, no `setcap`,
no `uinput`/udev, no `input` group) on a Mutter/GNOME Wayland session, and what
remains to be done for full Wayland-native coverage.

## TL;DR — Updated 2024-08 Ubuntu fix (Funput-style IBus)

- vietc auto-detects the session: **IBus engine first on GNOME Wayland** (Ubuntu
  default), then `zwp_input_method_v2`, then X11/XWayland.
- On **Ubuntu GNOME Wayland** (`WAYLAND_DISPLAY=wayland-0`, `DISPLAY=:0`, Mutter
  without `zwp_input_method_v2`) vietc now runs as a **native IBus engine**
  (`org.freedesktop.IBus` over D-Bus) — rootless, no privileges, covers **every**
  app including native Wayland `Firefox`/`GNOME Text Editor`/`Ptyxis` (which the
  X11 path cannot). This mirrors Funput's Linux strategy: be an IBus engine,
  don't fight it.
- On non-GNOME or X11 sessions it still uses the **rootless X11 path** or
  privileged `evdev`+`uinput` as before.
- When not using IBus, vietc **stops IBus on start** and **restarts IBus on clean exit**.
- Auto-start is handled by a systemd **user** service (`vietc.service`) with no
  `ConditionEnvironment=DISPLAY` so it starts on Wayland-native sessions too.

## Path selection (in `daemon/src/main.rs`)

 0. If `controller_mode` is set, run as Bamboo aux-controller (switches IBus
    engines per-app, no own composition).
 1. If `config::should_use_ibus_engine()` is true (**auto `true` on Ubuntu/Debian
    GNOME Wayland** where `zwp_input_method_v2` is absent — `auto_ibus=true`),
    run the **native IBus engine** (`daemon/src/ibus_engine.rs`). This is the
    default on Ubuntu and covers every app (X11/XWayland + native Wayland) via
    the compositor-approved `org.freedesktop.IBus` D-Bus path. Takes precedence
    and does NOT stop `ibus-daemon`. Toggle with `Ctrl+Space` and method with
    `Ctrl+Shift`; preedit is now visible (smooth, like Funput).
 2. Otherwise, build Wayland registry; if `zwp_input_method_manager_v2` is present,
    use the `zwp_input_method_v2` input-method path (true Wayland-native, rootless).
 3. Otherwise, if the keyboard devices are accessible (`input` group or root —
    i.e. `open_keyboard_devices()` succeeds), use the **evdev grab** path:
    - vietc grabs the physical keyboard (`EVIOCGRAB`), so the original
      keystrokes are suppressed and composition is clean. This covers **both
      X11 and Wayland-native apps** (the grab is at the kernel level, before the
      compositor routes input).
    - injection: `uinput` virtual keyboard (`protocol/src/x11_inject.rs`).
 4. Otherwise (no `input` group / no root, but `DISPLAY` set), fall back to the
     **rootless X11 keymap path** (`X11KeymapCapture` polling `XQueryKeymap` +
    `X11Injector` via `XTEST`) — X11/XWayland windows only; Wayland-native apps
    are not covered by this fallback.

Historically `DISPLAY` caused the evdev grab to be skipped entirely; on modern
Ubuntu the IBus path above is preferred because `evdev` grab receives no events
when the Wayland compositor owns the seat, and X11 path cannot see native
Wayland clients.

## IBus auto stop / restart

- `stop_ibus()` runs `ibus exit` when vietc starts (only once, guarded by the
  `IBUS_STOPPED` `AtomicBool`).
- An `IbusRestartGuard` (`daemon/src/main.rs`) is created at the very start of
  `main()`. Its `Drop` impl calls `restart_ibus()` (`ibus-daemon -d`), so IBus
  comes back whenever vietc exits **cleanly** (SIGINT/SIGTERM that the daemon
  handles and returns from `main`).
- `daemon/src/signal.rs` only *sets* `SIGNAL_EXIT` (it does not call
  `std::process::exit`), so `Drop` runs on a handled signal. `run_stdin_mode`
  (`daemon/src/stdin.rs`) also checks `SIGNAL_EXIT` and returns, otherwise the
  process would loop forever after the X11 capture returns and never restart
  IBus.
- SIGKILL bypasses `Drop`, so force-killing vietc will NOT restart IBus (by
  design — only a clean exit restores it).

## Known blocker: Mutter lacks `zwp_input_method_v2` — Solved via IBus

Enumerating globals on this session shows `zwp_text_input_manager_v3` but
**not** `zwp_input_method_manager_v2`. Mutter does not currently implement the
input-method-unstable-v2 protocol, so the pure Wayland-native `v2` path cannot
activate and previously vietc fell back to X11-only.

**Fix (Funput-style):** On Ubuntu GNOME this is no longer a blocker. When
`auto_ibus=true` (default) vietc auto-selects the **IBus engine path** (point 1
above) which uses the compositor-approved `org.freedesktop.IBus` interface that
GNOME already runs — the same interface GNOME Shell itself uses to talk to IBus.
IBus routes keystrokes from *every* client (X11/XWayland + native Wayland) to
vietc, so Wayland-native `Firefox`/`GNOME Text Editor`/`Ptyxis` are now covered.

Consequences **before** the fix (X11-only fallback):

- Covered **X11 / XWayland windows** only.
- Did **NOT** cover Wayland-native clients (GTK4/Qt on native Wayland).
- Required `ibus exit` hack which broke other languages.

**After** the fix: IBus path covers all clients; no `ibus exit`, no `input`
group, no `setcap` needed on Ubuntu.

### What is already ready for the v2 path

`daemon/src/wayland_im.rs` implements the full `zwp_input_method_v2` +
`zwp_virtual_keyboard_v1` flow, including a pure `plan_char()` mapping with unit
tests (8 tests passing). When Mutter (or any other compositor) exposes
`zwp_input_method_manager_v2`, vietc will pick it up automatically — no code
change needed for the daemon. The only requirement is the protocol being
advertised by the compositor.

### Future work for full Wayland-native coverage

- Track Mutter / GNOME Shell: once `zwp_input_method_manager_v2` is exposed,
  verify the v2 path activates here and re-run the `wayland_im` integration.
- Optionally add a periodic re-probe: if vietc starts before the compositor
  advertises v2, hotplug-detect when it appears and switch from X11 to v2
  without a restart. (Nice-to-have; currently vietc decides once at startup.)
- If a compositor only offers `zwp_text_input_manager_v3` (Mutter's current
  v3), note that v3 is client-side (apps opt in) and cannot be driven by an
  external daemon the way v2 can — so v2 (or the X11 path) is required for
  daemon-based IME.

## Auto-start (systemd user service)

File: `vietc.service` (repo root, also generated by `install.sh` into
`/usr/lib/systemd/user/vietc.service`):

```
[Unit]
Description=Viet+ Vietnamese IME Daemon (rootless)
PartOf=graphical-session.target
After=graphical-session.target
# No ConditionEnvironment=DISPLAY — IBus path needs no DISPLAY and must
# start on Wayland-native sessions too.

[Service]
Type=simple
ExecStart=/usr/bin/vietc-daemon
Restart=on-failure
RestartSec=3
KillMode=process
EnvironmentFile=-/etc/default/vietc

[Install]
WantedBy=graphical-session.target
```

- `KillMode=process` is important: the `IbusRestartGuard` spawns `ibus-daemon`
  in the service cgroup, and the default `control-group` kill mode would kill
  that IBus when the service stops. `process` keeps IBus alive after a clean
  exit/stop.
- `After=graphical-session.target` + `ConditionEnvironment=DISPLAY` ensure the
  daemon starts only once the session provides `DISPLAY` (XWayland), so it never
  falls into the privileged evdev path by accident.
- `Restart=on-failure` brings it back if it crashes; a clean exit (e.g.
  `systemctl --user stop`) restarts IBus via the guard.

Enable / run (as the user, not root):

```sh
systemctl --user daemon-reload
systemctl --user enable --now vietc.service
# verify
journalctl --user -f -u vietc.service   # look for "X11 keymap capture active"
# disable if needed
systemctl --user disable --now vietc.service
```

If your compositor does not import `DISPLAY` into the user manager, run once:

```sh
systemctl --user import-environment DISPLAY WAYLAND_DISPLAY
```

## Installing (rootless)

`vietc-setup.sh` (run as root) now:

1. Purges alternative IMEs (ibus-unikey, fcitx5, …).
2. Installs runtime/build deps (`libxkbcommon-dev` for the linker, `libx11-6`,
   `libxtst6`, `libwayland-client0`, `xclip`, `wl-clipboard`, …).
3. Copies binaries to `/usr/bin` (`vietc-daemon`, `vietc-cli`, `vietc-uinputd`,
   `vietc-tray`, `vietc-vk`).
4. **Skips** the old `setcap`/udev/`uinput` steps — not needed for rootless.
5. Installs the `vietc.service` user unit; user enables it as above.

> Build note: the linker needs `libxkbcommon.so`; the built binary records the
> SONAME `libxkbcommon.so.0`, so only stock `libxkbcommon0` is required at
> runtime. Build with `RUSTFLAGS="-L/tmp/xkblib"` where `/tmp/xkblib/libxkbcommon.so`
> -> `/usr/lib/x86_64-linux-gnu/libxkbcommon.so.0`.

## Limitations & caveats

- X11/XWayland-only coverage on this Mutter session (see blocker above).
- `vietc-vk` test tool still needs root for `uinput`; rootless users can skip it.
- Killing vietc with SIGKILL leaves IBus stopped (no clean `Drop`).

## Next steps (handoff)

1. **Verify on next login** that `vietc.service` starts, stops IBus, and input
   works in an X11 app (`GDK_BACKEND=x11 gnome-text-editor`).
2. **Watch for Mutter v2**: when GNOME exposes `zwp_input_method_manager_v2`,
   confirm the v2 path activates (`[vietc-wayland]` in logs) and re-test
   Wayland-native apps.
3. **Optional tray**: `vietc-tray` provides a menu/status icon and spawns the
   daemon (rootless here, since `grab` defaults to false). Run it manually if
   desired; it won't double-spawn because it detects the running daemon.
4. **Packaging**: the `packaging/deb` and `install.sh` still assume the
   privileged path; align them with the rootless `vietc.service` if shipping.
