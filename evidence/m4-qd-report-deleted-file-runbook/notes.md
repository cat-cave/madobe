# m4-qd-report-deleted-file-runbook Notes

- Documented the completion report `changedFiles[]` path contract in `docs/QD_WORKFLOW.md` next to the existing
  `qd complete --from-report` readiness rules.
- Pointed orchestrators in `docs/ORCHESTRATION.md` to verify that contract before running `qd complete --from-report`.
- The documented rule matches the existing qd report validator behavior: plain entries are existing repo-relative
  paths, while deleted files use `deleted:<repo-relative-path>` markers that are non-empty, non-absolute,
  traversal-free, and absent from the checkout.
- This is a documentation-only runbook node; no product runtime or platform validation is required.
