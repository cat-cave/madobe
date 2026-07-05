# m4-qd-report-required-content-contract Notes

## Scope

This node tightens `scripts/qd-reports-check.sh` so qd completion and audit reports cannot satisfy required content fields with empty strings or empty required arrays.

## Validation Boundary

This is repository metadata validation only. It does not change product runtime, capture, encode, transport, decode, render, presentation, cross-device, or latency behavior.

## Negative Smoke

`negative-smoke-empty-summary.log` records a temporary workspace that symlinked the repository root entries, copied `reports/`, blanked one copied completion report `summary`, and confirmed `scripts/qd-reports-check.sh` rejected it with a file-scoped error.
