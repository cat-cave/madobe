# m3-mac-video-frame-pipeline DAG proposal

Mac has no claimable nodes after `m3-audit`; the checked-in qd state is
42/42 done. The next useful Mac-side node is a local client pipeline proof that
bridges existing completed pieces:

- `m3-mac-video-receive-queue`
- `m3-videotoolbox-decode-sample`
- `m3-metal-renderer-skeleton`

The proposed node is intentionally local and does not require a new live
cross-device run. It should turn the current separate queue, decode, and render
proofs into one Mac-side queue-to-decode-to-render path with stable evidence.

Validation boundary:

- Claims local queue/decode/render pipeline behavior on macOS.
- Does not claim live network, product QUIC, cross-device presentation, or
  latency.
