# m4-qd-report-changed-file-contract notes

This node is limited to qd report metadata validation.

- Plain `changedFiles[]` entries now mean repo-relative paths that exist in the checkout.
- Deleted files must be represented with the `deleted:` prefix.
- `deleted:` entries require a non-empty repo-relative path without `..` traversal and are accepted only when the referenced path is absent from the checkout.
- No product runtime, capture, encode, transport, decode, render, presentation, cross-device, or latency behavior is changed or claimed.
