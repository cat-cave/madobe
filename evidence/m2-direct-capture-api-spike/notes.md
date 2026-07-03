# m2-direct-capture-api-spike Notes

Captured on the reference Linux host on July 3, 2026.

## Result

Status: implementation path identified.

The viable non-portal path is a small Wayland client built on
`ext_output_image_capture_source_manager_v1` plus `ext_image_copy_capture_manager_v1`, targeting the generated
Hyprland output by `wl_output` name.

The active compositor exposes the required direct-capture globals:

- `ext_output_image_capture_source_manager_v1` version 1.
- `ext_image_copy_capture_manager_v1` version 1.
- `zwp_linux_dmabuf_v1` version 5.
- `wp_linux_drm_syncobj_manager_v1` version 1.
- `wp_presentation` version 2, `CLOCK_MONOTONIC`.

The protocol contract can provide the metadata needed by `m2-capture-one-frame`: frame size from `buffer_size`,
DMA-BUF format/modifier choices from `dmabuf_format`, a monotonic frame timestamp from `presentation_time`, damage
from `damage`, and frame readiness from `ready`.

## Live Probe

A node-scoped generated output named `madobe-qd-m2-direct-capture-api-spike` was created, configured with
`hl.monitor` to `1280x720@60` at `50000x50000`, and removed after probing.

Evidence:

- `hyprland-monitors-with-output.json` proves the generated output existed at `1280x720@60`, scale 1, with current
  Hyprland format `XRGB8888`.
- `wayland-output-generated-source.txt` proves the generated output appears as a `wl_output` with name
  `madobe-qd-m2-direct-capture-api-spike`, version 4, and mode `1280x720@60`.
- `direct-grim-ppm-capture.txt` proves the legacy direct screenshot path can target the generated output by name with
  the live Wayland socket.
- `hyprland-monitors-after-remove.json` and `hyprland-monitors-after-wayland-output-probe.json` prove the node output
  was removed.

## Tool Findings

`grim` is useful only as a bounded targetability check. It emits a PNG/PPM image and does not expose DMA-BUF format,
modifier, presentation timestamp, or sync metadata to the caller.

No ready-made local CLI for `ext-image-copy-capture-v1` was found. `wl-screenrec`, `wayshot`, `wf-recorder`,
`grimshot`, `hyprshot`, and `hyprpicker` were missing from the plain PATH and the repo Nix shell.

## Implementation Path For m2-capture-one-frame

Implement `m2-capture-one-frame` as a direct Wayland client:

1. Connect to the live Wayland socket and enumerate globals.
2. Bind `wl_output` version 4 and select the output with `name == madobe-qd-<node-or-session-id>`.
3. Bind `ext_output_image_capture_source_manager_v1` and call `create_source(wl_output)`.
4. Bind `ext_image_copy_capture_manager_v1` and call `create_session(source, options = 0)`.
5. Wait for constraint events through `done`: record `buffer_size`, `dmabuf_device`, and one `dmabuf_format` entry.
6. Allocate a DMA-BUF on the advertised device with the selected DRM fourcc/modifier, matching width and height.
7. Create `wl_buffer` through `zwp_linux_dmabuf_v1`, preserving fd, plane offset, stride, fourcc, and modifier
   metadata.
8. Create a frame, attach the buffer, damage the full buffer for the first capture, and call `capture`.
9. On `presentation_time`, record the monotonic presentation timestamp; on `damage`, record damage rectangles.
10. On `ready`, publish `CapturedFrameMetadata` with `CaptureSync::Implicit` and note that no explicit sync fd was
    emitted by this protocol. If `failed`, surface the protocol failure reason.

The active `wp_linux_drm_syncobj_manager_v1` global should be recorded as host capability, but it does not change the
capture-frame sync metadata unless a later protocol or backend emits explicit fences for capture buffers.

## Disposition

This evidence can unblock `m2-capture-one-frame` for an implicit-sync direct capture implementation. It does not prove
an explicit sync-fd capture path; if the product requires explicit fence metadata rather than a sync-mode enum, add a
follow-up node for protocol or compositor work.
