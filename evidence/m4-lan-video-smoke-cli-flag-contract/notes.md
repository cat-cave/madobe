# m4-lan-video-smoke-cli-flag-contract

This node hardens the dependency-free LAN video smoke CLI parser. Value-taking
flags now reject another `--flag` token as a missing value instead of accepting
it as the value and continuing into network, sample, or artifact work.

Coverage includes missing and flag-like values for `send --addr`,
`send --sample`, `send --evidence-dir`, `receive --bind`, and
`receive --evidence-dir`. The focused tests assert typed
`SmokeErrorKind::Usage` failures and shared usage text for both send and receive.

This does not claim product QUIC, artifact-content validation, VideoToolbox
decode, Metal render, presentation, latency measurement, workflow dispatch,
credentials, Mac hardware, or cross-device validation.
