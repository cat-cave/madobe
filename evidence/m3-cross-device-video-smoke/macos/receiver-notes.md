# m3-cross-device-video-smoke macOS receiver notes

The macOS receiver was started on `0.0.0.0:47044` from branch
`spec/m3-cross-device-video-smoke` at commit
`ca33556522acd36ac97b355fc432824d84137e10`.

Receiver result:

- Status: passed.
- Local bind address: `0.0.0.0:47044`.
- Reachable Mac address posted to coordination issue: `192.168.1.15:47044`.
- Observed peer address: `192.168.1.15:52212`.
- Payload bytes: `84`.
- Payload SHA-256: `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`.
- Metadata validation: passed.
- Payload byte-count validation: passed.
- Payload SHA-256 validation: passed.

Important validation boundary:

- The receiver accepted and validated one AV1 sample.
- The observed peer address matches the Mac receiver host address, so macOS evidence alone does not prove the sender ran on the Linux host.
- Linux sender logs must be used as the authority for Linux host provenance and final cross-device status.
- This run does not claim product QUIC, VideoToolbox decode, Metal render, presentation, or latency.
