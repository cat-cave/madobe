# m2-native-dmabuf-allocation-proof Notes

Captured on the reference Linux host on July 3, 2026.

## Result

Status: complete.

The proof created a node-scoped Hyprland headless output named
`madobe-qd-m2-native-dmabuf-allocation-proof`, targeted it through the direct
`ext-image-copy-capture-v1` path, allocated a driver-native GBM DMA-BUF on the
advertised render node, imported it via `zwp_linux_dmabuf_v1`, and received
`ext_image_copy_capture_frame_v1.ready`.

The path used by the throwaway probe was:

`wl_output` named `madobe-qd-m2-native-dmabuf-allocation-proof` ->
`ext_output_image_capture_source_manager_v1.create_source` ->
`ext_image_copy_capture_manager_v1.create_session` ->
GBM allocation on `/dev/dri/renderD128` ->
`zwp_linux_dmabuf_v1.create_params` / `create` ->
`ext_image_copy_capture_frame_v1.attach_buffer` / `damage_buffer` / `capture`.

## Output Lifecycle

The generated output was created with:

`hyprctl output create headless madobe-qd-m2-native-dmabuf-allocation-proof`

It was configured with:

`hyprctl keyword monitor madobe-qd-m2-native-dmabuf-allocation-proof,1280x720@60,50000x50000,1`

Hyprland reported the created output as `1920x1080`, scale `2`, and `60 Hz` in
`hyprland-monitors-with-output.json`. This matches the capture session buffer
size used by the compositor. The output was removed after the probe, and
`hyprland-monitors-after-remove.json` contains no monitor with the node output
name.

## Allocator And Buffer

The capture session advertised DMA-BUF device `0xE280`, matched locally to
`/dev/dri/renderD128`.

The selected session format/modifier was:

- Format: `AR24`, DRM format code `875713089`.
- Modifier: `216172782120099856` (`0x0300000000606010`), an NVIDIA block-linear
  modifier advertised by the compositor.

The probe opened `/dev/dri/renderD128`, created a GBM device with backend
`nvidia`, and allocated the buffer with `gbm_bo_create_with_modifiers2` using
the advertised modifier. The allocated BO reported the same modifier, and the
same modifier was submitted to `zwp_linux_dmabuf_v1`.

Plane evidence:

- Planes: `1`.
- Offset: `0`.
- Stride: `7680`.
- Size: `1920x1080`.

The compositor accepted the buffer import:

- `bufferImport.created`: `true`.
- `bufferImport.failed`: `false`.

## Frame Result

The frame completed successfully:

- `ready`: `true`.
- `failed`: `false`.
- Timestamp: `CLOCK_MONOTONIC`, `tvSec=1033806`, `tvNsec=896411311`.
- Transform: `0`.
- Damage: full output, `0,0 1920x1080`.

This proves the previous dumb-buffer PRIME failure was allocator-specific, not
a fundamental direct-capture protocol blocker for this compositor/session.

## Dependencies

The plain host PATH and current repo `nix develop` shell are not sufficient for
building this native allocator probe:

- Plain PATH had `hyprctl` and `wayland-info`, but not `pkg-config`, `cc`, or
  `wayland-scanner`.
- `nix develop -c ...` provided `cc`, but `pkg-config` was missing, so probing
  `wayland-client`, `wayland-protocols`, `gbm`, `egl`, and `libdrm` stopped at
  `pkg-config: command not found`.

The proof used a temporary `nix-shell -p` environment, not a repo Nix edit:

`pkg-config wayland wayland-protocols libgbm libdrm libglvnd wayland-scanner gcc`

Versions resolved there:

- `wayland-client`: `1.25.0`.
- `wayland-protocols`: `1.48`.
- `gbm`: `26.0.3`.
- `egl`: `1.5`.
- `libdrm`: `2.4.133`.

For a repo-supported direct-capture allocator helper, the dev shell should add
at least `pkg-config`, `wayland-scanner`, `wayland`, `wayland-protocols`,
`libgbm`, and `libdrm`. EGL was not required by the GBM-only proof helper, but
`libglvnd` provides the checked `egl.pc` metadata if an EGL allocation/import
path is added later.

## qd Context

`qdcli` was not installed on this host. A read-only `qd status --json` attempt
through the installed `qd` binary was also not usable because the checked qd
database schema is older than that binary and requested `qd migrate`. No qd
migration or qd internal edit was performed.
