# m4-merge-group-trigger-dedupe Notes

## Scope

- Removed the redundant merge queue wrapper workflow.
- Kept `.github/workflows/ci.yml` as the single authoritative CI workflow.
- Preserved existing `ci.yml` triggers for `workflow_dispatch`, `workflow_call`, `pull_request`, `push` to `main`, and `merge_group`.
- Did not change action versions or CI job steps.

## Boundary

This node only changes GitHub Actions workflow routing and evidence.
It makes no product runtime, capture, encode, transport, decode, render,
presentation, cross-device, or latency claim.

## Validation

Recorded in `evidence/m4-merge-group-trigger-dedupe/commands.log`:

- `nix develop -c actionlint .github/workflows/*.yml`
- `nix develop -c just check`
- `git diff --check`
