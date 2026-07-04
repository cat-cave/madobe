# m4-qd-report-coverage-check Notes

This change extends `scripts/qd-reports-check.sh` to read `roadmap/qd-export.json`.

- Every `done` roadmap node now requires `reports/qd/<node-id>/completion.json`.
- Every `done` roadmap node now requires `reports/qd/<node-id>/audit.json`.
- Any report directory containing `completion.json` or `audit.json` must map to a roadmap node id.
- Report directories for real non-`done` nodes remain allowed, which keeps claimed or review nodes valid before final qd merge.

Validation was limited to repository metadata and qd report files. No product runtime, capture, encode, transport, decode, render, presentation, cross-device, or latency behavior is claimed.
