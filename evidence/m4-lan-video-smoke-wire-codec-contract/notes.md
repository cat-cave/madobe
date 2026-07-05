# m4-lan-video-smoke-wire-codec-contract

This node hardens the private LAN video smoke wire codec with platform-neutral
Rust unit tests. The tests use in-memory byte buffers only; they do not open
sockets, read the checked-in AV1 sample, require Mac hardware, or run live LAN
validation.

Coverage includes the stable fixed header layout, exact metadata and payload
round trip, lowercase SHA-256 metadata encoding, and typed failures for malformed
wire input or invalid metadata hash text.

This does not claim product QUIC, artifact-content validation, VideoToolbox
decode, Metal render, presentation, latency measurement, workflow dispatch,
credentials, or cross-device validation.

