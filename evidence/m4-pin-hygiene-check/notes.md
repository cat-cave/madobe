# m4-pin-hygiene-check Evidence

Implemented `scripts/pin-hygiene-check.sh` and wired it into
`just pin-hygiene-check` plus `just check`.

Validation:

- `nix develop -c just pin-hygiene-check`: passed.
- `nix develop -c just check`: passed.
- `nix develop -c shellcheck scripts/pin-hygiene-check.sh`: passed.
- Temporary negative smoke test with `.mise.toml` `rust = "latest"`: failed as expected.
- Temporary negative smoke test with `.mise.toml` `just = "stable"`: failed as expected.
- Temporary negative smoke test with workflow `uses: actions/checkout@v4`: failed as expected.
- Temporary positive smoke test with a 40-character commit SHA ref and `uses: ./actions/local`: passed.

The check accepts the repository's current full semver action refs, including
`actions/checkout@v7.0.0`, `cachix/install-nix-action@v31.10.6`, and
`jdx/mise-action@v4.2.0`.

No product runtime, capture, encode, transport, decode, render, presentation,
cross-device, or latency claim is made.
