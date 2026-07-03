# m2-capture-one-frame Notes

Captured on the reference Linux host on July 3, 2026.

## Result

Status: complete.

The proof captured one frame from the node-scoped Hyprland headless output
`madobe-qd-m2-capture-one-frame` through the direct Wayland capture path:

`wl_output` -> `ext_output_image_capture_source_manager_v1.create_source` ->
`ext_image_copy_capture_manager_v1.create_session` ->
GBM allocation on `/dev/dri/renderD128` ->
`zwp_linux_dmabuf_v1.create_params` / `create` ->
`ext_image_copy_capture_frame_v1.capture`.

The compositor emitted `ext_image_copy_capture_frame_v1.ready`, so this node
now has a completed frame with protocol-ready implicit synchronization.

## Frame Metadata

The successful capture is recorded in `direct-capture-frame.json`:

- Size: `1920x1080`.
- Buffer path: `linux-dmabuf/gbm-render-node`.
- Allocation API: GBM.
- Allocation node: `/dev/dri/renderD128`.
- Driver backend: `nvidia`.
- Format: `AR24`, DRM format code `875713089`.
- Modifier: `216172782120099856` / `0x0300000000606010`, NVIDIA block-linear.
- Planes: `1`.
- Stride: `7680`.
- Timestamp: `CLOCK_MONOTONIC`, `tvSec=1035024`, `tvNsec=423597405`.
- Transform: `0`.
- Damage: full output, `1920x1080`.
- Sync mode: `implicit/protocol-ready`.
- Explicit sync fd: not emitted by this protocol path.

No CPU readback is claimed.

## Output Lifecycle

The output was created, configured, captured, and removed. Cleanup evidence is in
`hyprland-monitors-after-remove.json`, which contains no monitor named
`madobe-qd-m2-capture-one-frame`.

## Dependency Note

This proof reused the native allocation path established by
`m2-native-dmabuf-allocation-proof`: a temporary proof environment with GBM,
Wayland protocol headers, libdrm, and wayland-scanner. The current repo dev
shell can run validation, but a committed native capture helper would need the
native graphics development packages documented by that prerequisite node.
