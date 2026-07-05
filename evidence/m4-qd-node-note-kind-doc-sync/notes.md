# m4-qd-node-note-kind-doc-sync

This node aligns `docs/QD_WORKFLOW.md` with the committed qd node-note
schema checker. The documented `node_notes[].kind` vocabulary is now
`blocker`, `retry`, and `note`, matching
`scripts/qd-report-check/roadmap-node-notes.sh`.

The docs clarify orchestration intent for each kind:

- `blocker`: durable missing-condition context.
- `retry`: failed attempts that should be retried or superseded.
- `note`: durable orchestration context such as verification sign-off details.

This is documentation-only and does not claim product runtime, capture, encode,
transport, decode, render, presentation, cross-device, latency, Mac hardware,
workflow dispatch, or credential-dependent validation.
