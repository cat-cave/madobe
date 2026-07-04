# m4-qd-report-shape-check Notes

This node adds a repository hygiene gate for committed qd completion and audit report shape under `reports/qd`.

The gate checks JSON parseability, required top-level fields, expected field types, `nodeId` directory matching,
accepted `realWorldValidation.status` values, empty completion `unverifiedItems`, and existence of plain repo-path
strings in completion `evidence` arrays.

The work is limited to repository metadata validation and does not make product runtime, capture, encode, transport,
decode, render, presentation, cross-device, or latency claims.
