# m4-product-quic-handoff-runbook Notes

This node hardens documentation for the blocked product QUIC cross-device smoke.
It does not run product QUIC, decode video, render with Metal, present frames, or
measure latency.

The documented product QUIC command syntax was read from PR #49's
`spec/m4-product-quic-cross-device-smoke` worktree because the command is not on
`main` until that draft PR lands. The runbooks require both orchestrators to use
the same PR branch or merged commit before executing the live smoke.
