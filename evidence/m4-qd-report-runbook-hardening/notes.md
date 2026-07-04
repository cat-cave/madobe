# m4-qd-report-runbook-hardening Notes

This node updates the qd workflow and orchestration runbooks with recent lifecycle lessons:

- Completion reports used with `qd complete --from-report` must not carry unresolved `unverifiedItems`.
- `realWorldValidation.status` must be `passed` or `not_required` at completion time.
- Hosted CI that is required validation should be recorded as qd CI evidence with `qd ci record-pass` before merge.
- Audit and finding disposition happen before CI promotion; CI evidence belongs in qd CI records and PR logs.
- qd export for a worktree branch should be run from the main checkout or qd root and written into the worktree's
  `roadmap/qd-export.json`.

No qd lifecycle commands were run for this node.
