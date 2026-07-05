# m4-lan-video-smoke-artifact-content-contract

This node hardens the checked-in LAN video smoke artifact writer contract with
platform-neutral Rust unit tests. The tests use synthetic sender and receiver
summaries, so they exercise `sender.log`, `receiver-listening.log`,
`receiver.log`, timelines, and `result.json` without opening sockets or
requiring a second machine.

The covered artifacts remain explicitly scoped to the TCP LAN smoke harness:
`transport=tcp`, `product_quic=false`, the checked-in AV1 sample byte count and
SHA-256, receiver validation booleans, and sequence-only timelines.

This does not claim live LAN validation, product QUIC, VideoToolbox decode,
Metal render, presentation, latency measurement, workflow dispatch, credentials,
or PR #49 changes.
