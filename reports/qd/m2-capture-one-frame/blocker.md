# m2-capture-one-frame Blocker

The direct Wayland probe did not complete a captured frame.

Evidence in `evidence/m2-capture-one-frame/direct-capture-frame.json` shows:

- The target `wl_output` was found.
- `ext_image_copy_capture_session_v1` constraints were received.
- DMA-BUF size, format, modifier, timestamp, damage, and attempted sync mode were recorded.
- Dumb allocation on the advertised `/dev/dri/renderD128` failed with permission denied in the first run.
- The fallback allocation on `/dev/dri/card1` succeeded and exported a PRIME fd.
- The frame emitted timestamp and damage, then ended with `frame.failed` reason `0` instead of `ready`.

Missing condition: `ext_image_copy_capture_frame_v1.ready` for the submitted Linux DMA-BUF.

Best next unblocker: allocate the capture buffer with a driver-native allocator that can produce an advertised NVIDIA
block-linear modifier, or update the compositor/protocol path so an implicit-modifier dumb-buffer PRIME fd is accepted.
