# m4-qd-worktree-report-path-doc-sync Notes

This node documents qd report-path handling for qd worktree branches.

The updated guidance says qd run report paths must remain repo-relative and must not point into `.qd/worktrees`.
When work happens in a qd worktree, the branch reports and evidence should be visible from the qd root at
`reports/...` and `evidence/...` while qd state is recorded; then the roadmap export is written back into the
worktree branch.

No qd behavior changed. No historical evidence or report records were rewritten. This node makes no product runtime,
capture, encode, transport, decode, render, presentation, cross-device, latency, Mac hardware, workflow dispatch, or
credential-dependent claim.
