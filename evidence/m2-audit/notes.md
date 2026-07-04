# m2-audit Notes

The audit split reviewed capture evidence, NVENC/unsafe boundary evidence, and downstream M3 readiness.

Blocking result:

- P0 findings: none.
- P1 findings: none.

Disposition summary:

- M2 capture evidence is sufficient for the narrow claim that one frame was captured from a node-scoped Hyprland
  output through `ext-image-copy-capture` using a GBM-allocated NVIDIA DMA-BUF with size, format, modifier,
  timestamp, damage, and implicit/protocol-ready sync recorded.
- The checked-in AV1 IVF sample is acceptable as a stable one-frame M3 fixture/decode-smoke input.
- The sample and M2 bundle do not prove streaming latency, product QUIC, HDR/color behavior, zero-copy
  capture-to-encode, no CPU readback after the grim-based encode sample, downstream Mac render/presentation, or
  cross-device behavior.
- The stale `m2-capture-one-frame/blocker.md` has been updated as superseded historical context.

Non-blocking follow-ups identified by the audit:

- Add validated newtypes or fallible constructors for productized encode settings before exposing runtime encode
  parameters broadly.
- Add a repo-supported direct capture helper/dev-shell dependency path before claiming reproducible product capture
  helper behavior.
- Require live M3 cross-device artifacts to directly prove decode/render/presentation/latency metrics before setting
  nonzero result metrics.
