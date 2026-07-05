# m4-qd-node-note-evidence-path-contract Notes

This node makes `node_notes[].evidence` durable in committed qd exports.

The two existing absolute blocker evidence paths were normalized from local qd worktree paths to canonical repo-relative `reports/qd/<node-id>/blocker.md` paths:

- `reports/qd/m2-capture-stream-proof/blocker.md`
- `reports/qd/m2-portal-screencast-client-proof/blocker.md`

The qd report checker now rejects absolute paths, `..` traversal, missing repo-relative paths, blank strings, and non-string non-null evidence values. HTTP(S) evidence URLs remain accepted so durable GitHub issue or PR links can still be recorded.

This is repository metadata and qd hygiene work. It makes no product runtime, capture, encode, transport, decode, render, presentation, cross-device, latency, Mac hardware, workflow dispatch, or credential-dependent claim.
