# m4-product-quic-result-artifact-reference-contract

This node hardens product QUIC `result.json` references.

It rejects non-portable or ambiguous evidence references before they can become qd evidence:

- absolute POSIX paths
- Windows drive-prefixed paths
- backslash-separated paths
- `..` traversal components
- unknown artifact `kind` values

The node is an artifact-schema hardening change only. It makes no live product QUIC, Mac decode/render, presentation,
latency, workflow-dispatch, or credential-dependent claim.
