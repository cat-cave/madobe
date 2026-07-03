# m2-capture-one-frame Notes

Captured on the reference Linux host on July 3, 2026.

## Result

Status: blocked after live direct-capture fallback.

The probe reached the required direct Wayland path:

`wl_output` named `madobe-qd-m2-capture-one-frame` ->
`ext_output_image_capture_source_manager_v1.create_source` ->
`ext_image_copy_capture_manager_v1.create_session` ->
`ext_image_copy_capture_frame_v1.capture`.

The compositor returned capture constraints and frame metadata, but did not produce a completed frame. The frame ended
with `ext_image_copy_capture_frame_v1.failed` reason `0` (`unknown`) instead of `ready`, so this node cannot claim a
successful one-frame capture or protocol-ready synchronization.

## Output Identity

The node-scoped Hyprland output was created as `madobe-qd-m2-capture-one-frame` and was visible through both Hyprland
monitor state and Wayland `wl_output` inventory. Hyprland reported the configured output at `1920x1080`, scale `2`, and
`60 Hz`. The output was removed after probing, and the cleanup snapshot contains no monitor with the node output name.

## Buffer Fallback Attempt

The capture session advertised DMA-BUF constraints:

- Size: `1920x1080`.
- Advertised DMA-BUF device: `/dev/dri/renderD128`, matching Wayland DMA-BUF main device `0xE280`.
- Candidate format: `AR24`, DRM format code `875713089`.
- Candidate modifier: `72057594037927935`, `DRM_FORMAT_MOD_INVALID` / implicit modifier.

Initial dumb-buffer allocation on the advertised render node failed with `Permission denied` in the first probe run. The
fallback then attempted dumb allocation on primary card nodes and succeeded on `/dev/dri/card1`. The resulting DRM dumb
buffer was exported as a PRIME fd and submitted through `zwp_linux_dmabuf_v1`.

The compositor accepted enough of the frame request to emit:

- Timestamp: `CLOCK_MONOTONIC`, `tvSec=1031971`, `tvNsec=127485650`.
- Damage: full output, `1920x1080`.
- Transform: `0`.

It then returned `failed` reason `0` instead of `ready`.

## Blocker

The missing condition is a successful `ext_image_copy_capture_frame_v1.ready` event for the submitted Linux DMA-BUF. The
best next unblocker is to allocate the buffer through a driver-native allocator that can produce one of Hyprland's
advertised NVIDIA block-linear modifiers, or to adjust the compositor/protocol path so an implicit-modifier PRIME fd
created from a dumb primary-node buffer is accepted for capture.

No CPU readback is claimed.
