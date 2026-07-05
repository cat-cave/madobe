# m4-qd-ready-queue-visibility Notes

Implemented a Linux-local Bash/JQ ready-queue diagnostic that reads `roadmap/qd-export.json` by default and does not
mutate qd state.

The current export has four raw `status == "ready"` nodes and zero assignable nodes after transitive `requires`
evaluation. The report points the M4 held chain at `m4-product-quic-cross-device-smoke`, which is blocked on Mac
receiver evidence for the product QUIC cross-device smoke.

Validation is repository tooling only. No product runtime, Mac hardware, workflow dispatch, cross-device validation,
or PR #49 change was performed or claimed.
