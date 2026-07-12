# vietc-vk

Standalone **virtual-keyboard test tool** for [Viet+ (vietc)](https://github.com/vndangkhoa/vietc) —
a Vietnamese input method for Linux.

It creates a real `/dev/uinput` virtual keyboard. vietc (already running, grabbing all
keyboards) intercepts those keystrokes, converts them per its VNI/Telex config, and re-injects the
result via its own (ignored) uinput device. This is exactly the path vietc's own integration test
(`daemon/tests/daemon_suite.rs`) uses.

## Features
- **On-screen keyboard** — click keys to send keystrokes via the virtual keyboard; focus any app
  (e.g. gedit) to watch vietc convert them live.
- **Run self-test** — types a built-in VNI/Telex dictionary through the virtual keyboard, reads the
  system clipboard, and reports PASS/FAIL per case.

## Build
```bash
cargo build --release
```

## Run (requires vietc daemon running + /dev/uinput access)
```bash
# one-time, as root: grant CAP_DAC_OVERRIDE so the tool can create a uinput device
sudo setcap cap_dac_override+ep target/release/vietc-vk

# ensure vietc itself is set up and running (see vietc repo / PLAN.md)
sudo bash /tmp/vietc-setup.sh
systemctl --user enable --now vietc.service

# launch the test tool
./target/release/vietc-vk
```

## How the self-test verifies
For each case the tool: clears the clipboard → types the raw keystrokes via the virtual keyboard
(~5 ms/key, ~1.5 s settle) → reads the clipboard with `wl-paste` (Wayland) / `xclip` (X11) →
asserts the expected Vietnamese is present. This mirrors vietc's proven integration test.

## License
MIT (same as vietc).
