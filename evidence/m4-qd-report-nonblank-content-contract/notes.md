# m4-qd-report-nonblank-content-contract

This node tightens qd report content validation from non-empty strings to non-blank strings after whitespace trimming.

Implementation notes:

- `scripts/qd-reports-check.sh` defines a shared jq `nonblank` predicate through the existing `expect_jq` helper.
- Completion report validation uses `nonblank` for `nodeId`, `summary`, `changedFiles[]`, `acceptanceEvidence[].criterion`, `acceptanceEvidence[].status`, `acceptanceEvidence[].evidence`, `commandsRun[].command`, `commandsRun[].status`, `commandsRun[].evidence`, `evidence[]`, and `realWorldValidation.evidence`.
- Audit report validation uses `nonblank` for `nodeId`, `acceptanceReviewed[].criterion`, `acceptanceReviewed[].status`, `acceptanceReviewed[].evidence`, and `realWorldValidation.evidence`.
- Real-world validation is not required because this is repository metadata validation only.
