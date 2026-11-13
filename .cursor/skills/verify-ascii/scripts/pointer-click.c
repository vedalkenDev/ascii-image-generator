/* Click at a compositor layout coordinate via wlr-virtual-pointer.
 * Usage: pointer-click X Y EXTENT_X EXTENT_Y
 */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <wayland-client.h>
#include "wlr-virtual-pointer-unstable-v1-client-protocol.h"

#ifndef BTN_LEFT
#define BTN_LEFT 0x110
#endif

static struct zwlr_virtual_pointer_manager_v1 *manager;
static struct wl_output *output;

static void registry_global(void *data, struct wl_registry *registry, uint32_t name,
                            const char *iface, uint32_t version) {
    (void)data;
    if (strcmp(iface, zwlr_virtual_pointer_manager_v1_interface.name) == 0) {
        uint32_t v = version < 2 ? version : 2;
        manager = wl_registry_bind(registry, name, &zwlr_virtual_pointer_manager_v1_interface, v);
    } else if (strcmp(iface, wl_output_interface.name) == 0 && output == NULL) {
        output = wl_registry_bind(registry, name, &wl_output_interface, 1);
    }
}

static void registry_remove(void *data, struct wl_registry *registry, uint32_t name) {
    (void)data;
    (void)registry;
    (void)name;
}

static const struct wl_registry_listener registry_listener = {
    .global = registry_global,
    .global_remove = registry_remove,
};

static uint32_t now_ms(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint32_t)(ts.tv_sec * 1000u + ts.tv_nsec / 1000000u);
}

int main(int argc, char **argv) {
    if (argc != 5) {
        fprintf(stderr, "usage: pointer-click X Y EXTENT_X EXTENT_Y\n");
        return 2;
    }
    uint32_t x = (uint32_t)atoi(argv[1]);
    uint32_t y = (uint32_t)atoi(argv[2]);
    uint32_t ex = (uint32_t)atoi(argv[3]);
    uint32_t ey = (uint32_t)atoi(argv[4]);
    if (ex == 0 || ey == 0) {
        fprintf(stderr, "extent must be > 0\n");
        return 2;
    }

    struct wl_display *display = wl_display_connect(NULL);
    if (!display) {
        fprintf(stderr, "pointer-click: no WAYLAND_DISPLAY\n");
        return 1;
    }
    struct wl_registry *registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &registry_listener, NULL);
    wl_display_roundtrip(display);
    if (!manager) {
        fprintf(stderr, "pointer-click: compositor has no zwlr_virtual_pointer_manager_v1\n");
        return 1;
    }

    struct zwlr_virtual_pointer_v1 *ptr;
    uint32_t ver = wl_proxy_get_version((struct wl_proxy *)manager);
    if (ver >= 2) {
        ptr = zwlr_virtual_pointer_manager_v1_create_virtual_pointer_with_output(manager, NULL, output);
    } else {
        ptr = zwlr_virtual_pointer_manager_v1_create_virtual_pointer(manager, NULL);
    }
    if (!ptr) {
        fprintf(stderr, "pointer-click: create_virtual_pointer failed\n");
        return 1;
    }

    uint32_t t = now_ms();
    zwlr_virtual_pointer_v1_motion_absolute(ptr, t, x, y, ex, ey);
    zwlr_virtual_pointer_v1_frame(ptr);
    wl_display_roundtrip(display);

    t = now_ms();
    zwlr_virtual_pointer_v1_button(ptr, t, BTN_LEFT, WL_POINTER_BUTTON_STATE_PRESSED);
    zwlr_virtual_pointer_v1_frame(ptr);
    wl_display_roundtrip(display);

    t = now_ms();
    zwlr_virtual_pointer_v1_button(ptr, t, BTN_LEFT, WL_POINTER_BUTTON_STATE_RELEASED);
    zwlr_virtual_pointer_v1_frame(ptr);
    wl_display_roundtrip(display);

    zwlr_virtual_pointer_v1_destroy(ptr);
    zwlr_virtual_pointer_manager_v1_destroy(manager);
    if (output) {
        wl_output_destroy(output);
    }
    wl_registry_destroy(registry);
    wl_display_flush(display);
    wl_display_disconnect(display);
    return 0;
}
