#define _GNU_SOURCE
#include <dlfcn.h>
#include <signal.h>
#include <X11/Xlib.h>

int XFreeCursor(Display *dpy, Cursor cursor) {
	int (*real_free)(Display *, Cursor) = dlsym(RTLD_NEXT, "XFreeCursor");
	/* XDefineCursor is still buffered. Sync, then stop before xsetroot
	   closes the display. Xvfb drops the cursor on disconnect. */
	XSync(dpy, False);
	raise(SIGSTOP);
	return real_free(dpy, cursor);
}
