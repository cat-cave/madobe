# Pull Request

## Summary

- TBD

## qd Evidence

- Node id: `TBD`
- Acceptance criteria:
  - TBD
- Evidence paths:
  - `evidence/<node-id>/commands.log`
  - `evidence/<node-id>/notes.md`
- qd reports:
  - Completion: `reports/qd/<node-id>/completion.json`
  - Audit: `reports/qd/<node-id>/audit.json` or `not yet audited`
- Platform validation status:
  - Required platform checks: `passed` / `not_required` / `blocked`
  - Cross-device validation, when relevant: `passed` / `not_required` /
    `blocked`
  - Opposite-platform validation, when relevant: `passed` / `not_required` /
    `blocked`
- Residual risks:
  - TBD
- DAG or topology changes:
  - `none` or describe proposed qd node/edge changes

## Verification

- [ ] Linux PR readiness: `nix develop -c just ci-local`
      (`direct-capture-preflight` runs before `verify` and `coverage`)
- [ ] Targeted Linux hygiene, when useful: `nix develop -c just check`
- [ ] Native macOS validation, when relevant: `just macos-check`
- [ ] qd evidence and reports are linked above
- [ ] DAG or topology changes are recorded above

## Risk

- TBD
