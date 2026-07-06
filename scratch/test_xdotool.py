from Xlib import X, display
import subprocess
import threading
import time
import sys

d = display.Display()
root = d.screen().root
window = root.create_window(
    10, 10, 100, 100, 1,
    d.screen().root_depth,
    X.InputOutput,
    X.CopyFromParent,
    background_pixel=d.screen().white_pixel,
    event_mask=X.KeyPressMask | X.KeyReleaseMask
)
window.set_wm_name("Test Window")
window.map()
d.sync()

def run_xdotool():
    time.sleep(1.0)
    # Focus the window using xdotool search
    subprocess.run(["xdotool", "search", "--name", "Test Window", "windowactivate"])
    time.sleep(0.5)
    # Type 'hello' first
    subprocess.run(["xdotool", "type", "hello"])
    time.sleep(0.5)
    # Run the chained command and capture stdout/stderr
    res = subprocess.run(["xdotool", "key", "BackSpace", "BackSpace", "type", "--clearmodifiers", "world"], capture_output=True, text=True)
    print(f"Subprocess status: {res.returncode}", flush=True)
    print(f"Subprocess stdout: {res.stdout}", flush=True)
    print(f"Subprocess stderr: {res.stderr}", flush=True)
    time.sleep(0.5)
    # Exit process
    import os
    os._exit(0)

threading.Thread(target=run_xdotool, daemon=True).start()

while True:
    event = d.next_event()
    if event.type == X.KeyPress:
        keysym = d.keycode_to_keysym(event.detail, 0)
        print(f"KeyPress: {keysym}", flush=True)
