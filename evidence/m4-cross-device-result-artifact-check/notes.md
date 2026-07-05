# m4-cross-device-result-artifact-check Notes

This node adds a generic cross-device result artifact checker for the existing M3 cross-device video result shape.

The checker validates the checked-in generic fixture, rejects unsupported decode/render/presentation/latency claims without matching evidence artifact kinds, rejects invalid latency ordering, rejects unsafe artifact paths, rejects unknown artifact kinds, and verifies explicit result artifact references exist.

No live product QUIC, decode, render, presentation, latency, macOS validation, workflow dispatch, or credential-dependent behavior is claimed by this node.
