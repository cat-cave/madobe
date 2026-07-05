# m4-qd-export-parity-check Notes

This node adds an orchestrator-local parity check for the qd cache and committed export.

The check is intentionally not wired into `just check`: hosted CI and ordinary worktrees do not carry qd's local
database. Orchestrators run it from the qd root when they need to prove that the live cache and a committed export are
identical, including branch exports under `.qd/worktrees/<node-id>/roadmap/qd-export.json`.

`qd sync --expect-clean` remains the required startup guard. The new check is a stronger byte-for-byte parity check
for suspicious drift or pre-export validation. Evidence includes a qd sync dry-run where node-note-only drift reports
`summary: "no drift"` and parity-check negative smokes that catch node-note and run drift.

This is repository metadata and qd hygiene work. It makes no product runtime, capture, encode, transport, decode,
render, presentation, cross-device, latency, Mac hardware, workflow dispatch, or credential-dependent claim.
