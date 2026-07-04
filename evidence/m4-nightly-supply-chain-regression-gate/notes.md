# m4-nightly-supply-chain-regression-gate

## Scope

- Added the existing `nix develop -c just security` supply-chain gate to the scheduled nightly `deep-checks` job.
- The new step runs after Nix setup and before the existing long-running `just coverage` and `just mutants` steps.
- Left `.github/workflows/ci.yml` unchanged, preserving pull request and main CI behavior.

## Validation

- `nix develop -c just security` passed.
- `nix develop -c actionlint .github/workflows/*.yml` passed.
- `nix develop -c just check` passed on retry after normalizing terminal control characters in this evidence transcript; the first attempt failed because the partially captured transcript caused a `typos` false positive in `commands.log`.
- `git diff --check` passed.

## Boundary

This node only changes scheduled workflow gate ordering and records validation evidence. It makes no product runtime, capture, encode, transport, decode, render, presentation, cross-device, or latency claim.
