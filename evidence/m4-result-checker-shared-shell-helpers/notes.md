# m4-result-checker-shared-shell-helpers Notes

This node consolidates shared shell helper logic for result checkers without changing product runtime behavior.

The new helper owns jq/tool availability, repository-root normalization, repo-relative artifact path predicates, non-negative integer predicates, and explicit artifact existence/content checks used by the generic cross-device checker.

Product QUIC and generic cross-device result checkers keep their schema-specific validation rules and negative fixtures. No live product QUIC, Mac hardware, cross-device transport, decode, render, presentation, latency, workflow dispatch, or credential-dependent behavior is changed or claimed.
