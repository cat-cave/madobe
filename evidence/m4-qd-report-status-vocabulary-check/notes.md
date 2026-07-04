# m4-qd-report-status-vocabulary-check Notes

This node extends the committed qd report checker so documented status vocabularies are enforced for completion and
audit report arrays.

The checker now constrains completion `acceptanceEvidence[].status` to `passed`, `failed`, or `not_required`;
completion `commandsRun[].status` to `passed`, `failed`, or `not-run`; and audit `acceptanceReviewed[].status` to
`passed`, `failed`, or `not_required`. The existing `realWorldValidation.status` constraint remains `passed` or
`not_required`.

The work is limited to repository report metadata validation and does not make product runtime, capture, encode,
transport, decode, render, presentation, cross-device, or latency claims.
