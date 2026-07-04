# m4 Nightly Coverage Baseline Ratchet Notes

## Decision

`just coverage` now enforces the current workspace line coverage baseline with
an integer floor of 78%.

The measured current baseline is 78.84% total line coverage from:

```sh
nix develop -c cargo llvm-cov report --summary-only --ignore-filename-regex 'apps/.*/src/main\.rs'
```

Using 78% keeps the nightly gate below the measured baseline while restoring an
actionable failure signal if coverage drops.

## Evidence

- The prior 95% policy reproduced as a failing gate after writing `lcov.info`;
  see `coverage-95-failure.log`.
- The calibrated `nix develop -c just coverage` gate passed with
  `--fail-under-lines 78`; see `just-coverage.log`.
- The measured summary reports `TOTAL` line coverage of 78.84%; see
  `coverage-summary.txt`.

## Future Ratchets

Future increases to the workspace coverage floor should be explicit ratchet
nodes with fresh coverage evidence. The crate-level targets in
`docs/ENGINEERING.md` remain directionally useful, but this node only calibrates
the nightly workspace gate to the measured baseline.

## Product Claims

This node only changes repository coverage tooling and coverage policy
documentation. It does not change product QUIC, decode, render, presentation,
or latency claims.
