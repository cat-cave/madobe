# m4-direct-capture-preflight-ci-gate Notes

This node wires the repo-supported direct capture helper build preflight into
hosted Linux CI only.

The Linux job now runs:

```sh
nix develop -c just direct-capture-preflight
```

after `nix flake check` and before the broader repo `just check`, test, and
security gates.

The macOS job remains unchanged and does not run the Linux/Nix direct capture
preflight.

Boundary: this is a build-environment and helper reproducibility gate. It does
not run runtime capture, product QUIC, portal capture, capture-to-NVENC,
zero-copy validation, cross-device validation, decode, render, presentation, or
latency validation.
