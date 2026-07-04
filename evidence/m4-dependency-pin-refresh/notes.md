# m4-dependency-pin-refresh Notes

## Scope

This node refreshes current dependency and tool pins only:

- `flake.lock` nixpkgs input was refreshed with `nix flake update`.
- `.mise.toml` Tuist pin was refreshed after `nix develop -c mise latest tuist` reported `4.201.0`.
- Evidence and qd completion report files were added for this node.

No flake input policy, product code, runtime behavior, protocol behavior, capture behavior, encode behavior, render behavior, macOS app behavior, or CI workflow behavior was changed.

## No Product Runtime Claim

Validation for this node is limited to repository/tooling checks that the refreshed pins still evaluate and pass local checks. This evidence does not claim any product runtime, real hardware, video path, QUIC, decode, render, latency, or cross-device behavior was validated.
