// vietc-xrecord: Captures keyboard events via XRecord (blocking mode)
// and writes fixed-size binary records to stdout.
// The parent vietc daemon reads events from the pipe.
//
// Pipe event format: 8 bytes, packed
//   keycode: u8    (0 for focus events)
//   pressed: u8    (1=press, 0=release, 0 for focus)
//   state:   u16   (modifier mask for keys, 1=FocusIn, 2=FocusOut)
//   pad:     [u8;4]
//
// Compile: gcc -O2 -o vietc-xrecord vietc-xrecord.c -lX11 -lXtst

#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <X11/Xlib.h>
#include <X11/extensions/record.h>

#pragma pack(push, 1)
typedef struct {
    unsigned char keycode;
    unsigned char pressed;
    unsigned short state;
    unsigned char padding[4];
} PipeEvent;
#pragma pack(pop)

static void write_event(const PipeEvent *ev) {
    const char *ptr = (const char *)ev;
    size_t remaining = sizeof(PipeEvent);
    while (remaining > 0) {
        ssize_t n = write(STDOUT_FILENO, ptr, remaining);
        if (n < 0) {
            if (errno == EINTR) continue;
            _exit(1);
        }
        ptr += n;
        remaining -= n;
    }
}

static void record_cb(XPointer closure, XRecordInterceptData *data) {
    if (data->category != XRecordFromServer)
        return;
    if (data->data_len < 2)
        return;

    unsigned char *ev = data->data;
    unsigned char event_type = ev[0];

    if (event_type != 2 && event_type != 3 &&
        event_type != 9 && event_type != 10)
        return;

    PipeEvent out;
    memset(&out, 0, sizeof(out));

    switch (event_type) {
        case 2: /* KeyPress */
            out.keycode = ev[1];
            out.pressed = 1;
            if (data->data_len >= 4)
                out.state = *(unsigned short *)(ev + 2);
            write_event(&out);
            break;

        case 3: /* KeyRelease */
            out.keycode = ev[1];
            out.pressed = 0;
            if (data->data_len >= 4)
                out.state = *(unsigned short *)(ev + 2);
            write_event(&out);
            break;

        case 9: /* FocusIn */
            out.keycode = 0;
            out.pressed = 0;
            out.state = 1;
            write_event(&out);
            break;

        case 10: /* FocusOut */
            out.keycode = 0;
            out.pressed = 0;
            out.state = 2;
            write_event(&out);
            break;

        default:
            break;
    }
}

int main(void) {
    Display *dpy = XOpenDisplay(NULL);
    if (!dpy) { fprintf(stderr, "vietc-xrecord: no display\n"); return 1; }

    int major = 0, minor = 0;
    XRecordQueryVersion(dpy, &major, &minor);

    XRecordRange *range = XRecordAllocRange();
    if (!range) { fprintf(stderr, "vietc-xrecord: XRecordAllocRange failed\n"); return 1; }
    range->device_events.first = KeyPress;
    range->device_events.last  = FocusOut;

    XRecordClientSpec spec = XRecordAllClients;
    XRecordContext ctx = XRecordCreateContext(dpy, 0, &spec, 1, &range, 1);
    XFree(range);
    if (!ctx) { fprintf(stderr, "vietc-xrecord: XRecordCreateContext failed\n"); return 1; }

    fprintf(stderr, "vietc-xrecord: ready (XRecord %d.%d, ctx=%ld)\n", major, minor, (long)ctx);
    fflush(stderr);

    /* BLOCK here — callback fires for each keyboard/focus event */
    XRecordEnableContext(dpy, ctx, record_cb, NULL);

    XCloseDisplay(dpy);
    return 0;
}
