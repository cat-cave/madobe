# m3-lan-video-smoke-harness Notes

This node adds a dependency-free TCP validation harness for the checked-in AV1
IVF sample at `evidence/m2-nvenc-encode-sample/sample-av1.ivf`.

The sender loads the checked-in sample, validates the IVF header and expected
SHA-256, sends deterministic frame metadata, then sends the IVF file as opaque
payload bytes. The receiver accepts one TCP connection and validates:

- metadata: AV1, 160x90, frame id 1, keyframe, SHA-256 metadata.
- payload byte count: 84 bytes.
- payload SHA-256:
  `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`.

The localhost CLI smoke passed after rerunning outside the socket-restricted
sandbox. The first sandboxed attempt failed with `Operation not permitted` on
localhost bind/connect, and that is recorded in `commands.log`.

## Non-Claims

This is only cross-device smoke preparation. It is explicitly:

- not QUIC.
- not the product transport.
- not VideoToolbox decode.
- not Metal render.
- not presentation evidence.
- not latency proof.
