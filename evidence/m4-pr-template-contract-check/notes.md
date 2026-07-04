# m4-pr-template-contract-check Notes

This node adds a repository check for the qd pull request template contract
described in `docs/ORCHESTRATION.md`.

The check validates stable PR template anchors for:

- qd node id
- acceptance criteria
- evidence paths
- qd completion and audit reports
- platform validation status
- cross-device and opposite-platform validation status when relevant
- residual risks
- DAG or topology changes
- Linux readiness
- macOS validation
- qd evidence/report linkage

The implementation is limited to repository hygiene tooling and node evidence.
It makes no product runtime, capture, encode, transport, decode, render,
presentation, cross-device, or latency claim.
