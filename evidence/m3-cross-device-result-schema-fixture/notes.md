# m3-cross-device-result-schema-fixture

This node adds a checked `m3-cross-device-video-smoke/result.json` fixture and validation path before the live
cross-device smoke.

Implemented coverage:

- The fixture includes `node_id`, `branch`, Linux and macOS commits, start/end timestamps, pass/fail state, metrics,
  artifacts, and notes.
- Latency fields are nullable when not measured.
- Nonzero decode, render, or presentation metrics require matching artifact roles.
- Non-null latency metrics require matching latency evidence.
- The checked fixture has zero frame metrics and null latency and makes no live cross-device behavior claim.

Validation evidence:

- `protocol-tests.log` shows the fixture validates and round-trips against the checked JSON bytes.
- `protocol-tests.log` also shows rejection coverage for invented decode, render, presentation, and latency values
  without matching evidence.
- `just-check.log` shows the workspace `just check` path passing.
