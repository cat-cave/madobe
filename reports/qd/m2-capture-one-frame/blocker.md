# m2-capture-one-frame Blocker

Superseded: the initial direct Wayland probe did not complete a captured frame, but a later GBM/native
DMA-BUF capture run did complete and is now the authoritative state for `m2-capture-one-frame`.

Initial evidence showed:

- The target `wl_output` was found.
- `ext_image_copy_capture_session_v1` constraints were received.
- DMA-BUF size, format, modifier, timestamp, damage, and attempted sync mode were recorded.
- Dumb allocation on the advertised `/dev/dri/renderD128` failed with permission denied in the first run.
- The fallback allocation on `/dev/dri/card1` succeeded and exported a PRIME fd.
- The frame emitted timestamp and damage, then ended with `frame.failed` reason `0` instead of `ready`.

The missing condition was `ext_image_copy_capture_frame_v1.ready` for the submitted Linux DMA-BUF.

Resolution evidence:

- `reports/qd/m2-capture-one-frame/completion.json` records a completed frame through
  `ext-image-copy-capture` using a GBM-allocated NVIDIA block-linear DMA-BUF on the advertised render node.
- `reports/qd/m2-capture-one-frame/audit.json` records `frame.ready=true` and `frame.failed=false`.
- `evidence/m2-capture-one-frame/direct-capture-frame.json` is the current authoritative artifact and records
  `success=true`, `frame.ready=true`, size, format, modifier, timestamp, damage, and implicit/protocol-ready sync.

This file is kept only as historical blocker context and must not be read as the current node state.
