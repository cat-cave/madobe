#include <gbm.h>
#include <stdint.h>
#include <stdio.h>
#include <wayland-client.h>
#include <xf86drm.h>

#include "ext-image-capture-source-v1-client-protocol.h"
#include "ext-image-copy-capture-v1-client-protocol.h"
#include "linux-dmabuf-v1-client-protocol.h"

static void touch_direct_capture_symbols(void) {
  (void)&wl_display_connect;
  (void)&wl_display_disconnect;
  (void)&ext_output_image_capture_source_manager_v1_interface;
  (void)&ext_image_copy_capture_manager_v1_interface;
  (void)&zwp_linux_dmabuf_v1_interface;
  (void)&gbm_create_device;
  (void)&gbm_bo_create_with_modifiers2;
  (void)&gbm_bo_get_modifier;
  (void)&drmGetDevices2;
}

int main(void) {
  touch_direct_capture_symbols();
  puts("direct capture build preflight ok");
  return 0;
}
