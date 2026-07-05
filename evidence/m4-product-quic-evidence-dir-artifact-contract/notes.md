# m4-product-quic-evidence-dir-artifact-contract

This node tightens product QUIC result evidence hygiene. Role-specific
artifacts are now bound to the endpoint evidence directory that owns them:
`sender_log` must live under `sender.evidenceDir`, while `receiver_log` and
`payload_validation_evidence` must live under `receiver.evidenceDir`.

The shell checker and Rust validator both use path-boundary-aware checks, so a
sibling path such as `linux-sender-extra/sender.log` does not satisfy
`linux-sender`.

The no-argument `just product-quic-result-check` remains schema-only for the
checked-in golden fixture. Explicit result validation still checks artifact
existence, non-empty core artifacts, and stable sender/receiver log tokens.

This node does not run live product QUIC, UDP/LAN connectivity, Mac hardware,
certificate handoff, decode, render, presentation, latency, workflow dispatch,
or credential-dependent validation.
