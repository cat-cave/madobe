# m3-quic-transport-skeleton Notes

This node adds the first transport skeleton as `madobe-transport`. The implementation is intentionally
dependency-free and in-memory only: `LoopbackQuicEndpoint::pair` creates two connected endpoints backed by local
queues. The public crate docs state that no socket I/O, live network behavior, cross-device validation, capture,
decode, render, or latency behavior is claimed.

Acceptance mapping:

- Loopback sender and receiver tests pass:
  `evidence/m3-quic-transport-skeleton/transport-tests.log`.
- Video lane carries framed sample payloads:
  `VideoSample` combines `EncodedVideoFrameMetadata` with opaque payload bytes, and `VideoLane::send/receive`
  transfers the sample through the loopback lane.
- Telemetry records send and receive timestamps:
  `TransportTelemetryLog` stores `TransportTelemetryEvent` values with session id, lane id, frame id, direction,
  and deterministic `madobe_telemetry::Timestamp` values.
- Transport errors are typed:
  `TransportErrorKind` covers payload size overflow, metadata/payload size mismatch, and unexpected lane envelopes.
- No live network claim is made:
  the crate is named and documented as a loopback skeleton, and the evidence only includes local loopback tests.

Dependency note: no third-party Rust dependency was added. `Cargo.lock` only changes to register the new local
workspace crate and its existing local path dependencies on `madobe-protocol` and `madobe-telemetry`.
