# m4-action-ref-strict-pin-check Evidence

Tightened `scripts/pin-hygiene-check.sh` so external GitHub Actions refs are
accepted only when pinned to a full `vMAJOR.MINOR.PATCH` semver tag, with simple
optional pre-release/build metadata, or a 40-character hex commit SHA.

Validation:

- `nix develop -c just pin-hygiene-check`: passed.
- `nix develop -c shellcheck scripts/pin-hygiene-check.sh`: passed.
- `nix develop -c just check`: passed.
- Temporary negative smoke test with `uses: actions/checkout@v4`: failed as
  expected.
- Temporary negative smoke test with `uses: actions/checkout@v4.2`: failed as
  expected.
- Temporary negative smoke test with `uses: actions/checkout@main`: failed as
  expected.
- Temporary positive smoke test with current full semver refs
  `actions/checkout@v7.0.0`, `cachix/install-nix-action@v31.10.6`, and
  `jdx/mise-action@v4.2.0`: passed.
- Temporary positive smoke test with a 40-character SHA ref, local action, and
  docker action: passed.
- `git diff --check`: passed.

No product runtime, capture, encode, transport, decode, render, presentation,
cross-device, or latency claim is made.
