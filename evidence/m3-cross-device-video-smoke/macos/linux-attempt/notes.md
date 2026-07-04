# m3-cross-device-video-smoke macOS Linux-attempt notes

This is the receiver evidence for the corrected Linux-to-Mac attempt after the earlier local-sender correction.

Receiver result:

- Status: passed.
- Local bind address: `0.0.0.0:47044`.
- Mac receiver address requested for Linux: `192.168.1.15:47044`.
- Observed peer address: `192.168.1.23:46968`.
- Payload bytes: `84`.
- Payload SHA-256: `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`.
- Metadata validation: passed.
- Payload byte-count validation: passed.
- Payload SHA-256 validation: passed.

Validation boundary:

- The receiver accepted and validated one AV1 sample from a peer distinct from the Mac receiver address.
- Linux sender evidence remains the authority for Linux host provenance and final cross-device aggregation.
- This run proves TCP LAN sample transfer only. It does not claim product QUIC, VideoToolbox decode, Metal render, presentation, or latency.
