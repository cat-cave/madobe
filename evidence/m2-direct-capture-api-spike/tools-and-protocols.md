# Candidate Tools And Protocols

## Local Tools

- `hyprctl`: present, Hyprland 0.55.2. Required for node-scoped headless output lifecycle.
- `grim`: present. Can target `-o madobe-qd-m2-direct-capture-api-spike` and produce PNG/PPM images, but exposes only encoded image metadata to the caller.
- `slurp`: present, region selector only; not useful for direct generated-output frame metadata.
- `wayland-info`: present. Confirms active Wayland globals and generated output discovery.
- `pipewire`, `pw-cli`, `pw-dump`: present, PipeWire 1.6.5. Adjacent to portal capture but no non-portal generated-output stream is created by simply adding the output.
- `wl-screenrec`, `wayshot`, `wf-recorder`, `grimshot`, `hyprshot`, `hyprpicker`: missing from both the plain PATH and the repo Nix shell.

## Active Direct-Capture Globals

- `zwlr_screencopy_manager_v1` version 3: available. This is the legacy path used by `grim`; the local tool surface does not expose DMA-BUF modifier, timestamp, or sync metadata.
- `ext_output_image_capture_source_manager_v1` version 1: available. Creates an `ext_image_capture_source_v1` from a `wl_output`.
- `ext_image_copy_capture_manager_v1` version 1: available. Its protocol advertises `buffer_size`, `dmabuf_device`, `dmabuf_format`, per-frame `presentation_time`, damage, and `ready`/`failed`.
- `zwp_linux_dmabuf_v1` version 5: available. Global feedback includes DRM fourcc/modifier pairs, including `XB24`/`AB24` with NVIDIA block-linear modifiers and linear/invalid entries for some formats.
- `wp_linux_drm_syncobj_manager_v1` version 1: available globally, but `ext-image-copy-capture-v1` does not carry syncobj/fence requests or events.
- `wp_presentation` version 2: available, `CLOCK_MONOTONIC`.

## Metadata Disposition

- Size: live-proven through Hyprland monitor JSON, `wl_output`, `grim` PPM header, and available as `ext_image_copy_capture_session_v1.buffer_size`.
- Format: not exposed by `grim`; exposed by `ext_image_copy_capture_session_v1.dmabuf_format`.
- Modifier: not exposed by `grim`; exposed by `ext_image_copy_capture_session_v1.dmabuf_format` modifier arrays.
- Timestamp: not exposed by `grim`; exposed by `ext_image_copy_capture_frame_v1.presentation_time`.
- Sync mode: no explicit sync fd is exposed by `ext-image-copy-capture-v1`; record this backend as implicit/protocol-ready synchronization when the frame `ready` event is received.
