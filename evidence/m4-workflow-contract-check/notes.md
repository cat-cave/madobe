# m4-workflow-contract-check Evidence

Implemented `scripts/workflow-contract-check.sh` and wired it into
`just workflow-contract-check` plus `just check`.

The check validates stable repository workflow invariants for:

- `.github/workflows/ci.yml` triggers, top-level permissions, Linux/macOS jobs,
  job timeouts, required actions, Rust component setup, and standard CI
  commands.
- `.github/workflows/nightly.yml` scheduled/manual triggers, top-level
  permissions, job timeout, and deep-check commands.

Missing required fields produce file-scoped errors through the shared
`report_error` helper.

No product runtime, capture, encode, transport, decode, render, presentation,
cross-device, or latency claim is made.
