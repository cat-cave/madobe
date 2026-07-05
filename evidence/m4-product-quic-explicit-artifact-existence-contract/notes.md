# m4-product-quic-explicit-artifact-existence-contract Notes

This node tightens product QUIC result evidence validation for explicit result files.

Default `just product-quic-result-check` remains schema-only for the checked-in golden fixture and generated negative schema fixtures. Explicit result-file mode now verifies referenced artifacts exist, and that core evidence kinds are non-empty when present.

Core evidence kinds checked for non-empty content are `commands_log`, `sender_log`, `receiver_log`, `payload_validation_evidence`, and `notes`.

No live product QUIC, Mac decode, render, presentation, latency, workflow dispatch, credential, or product runtime behavior is changed or claimed.
