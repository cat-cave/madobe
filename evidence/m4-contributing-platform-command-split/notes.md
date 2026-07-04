# m4 Contributing Platform Command Split Notes

## Scope

This node updates `CONTRIBUTING.md` so contributor onboarding follows the
established command model:

- `just` is the shared repository command surface.
- Linux readiness uses Nix as the environment boundary.
- macOS readiness uses native Xcode, Mise, and Homebrew commands.

## Platform Readiness

Linux onboarding now documents entering the shell with `nix develop` and
running the local PR readiness gate with `nix develop -c just ci-local`.

macOS onboarding now documents `just macos-bootstrap` followed by
`just macos-check`, and states that Nix is not required for macOS onboarding or
readiness checks.

## Policy Preservation

The generated-file policy remains intact: pull requests should keep generated
files out of version control unless the generator is unavailable in CI.

The Rust crate policy remains intact: new Rust crates must inherit the
workspace license, lints, edition, and Rust version.

## Product Claims

This node changes contributor onboarding documentation and qd evidence only. It
does not make or change product runtime, capture, encode, transport, decode,
render, presentation, cross-device, or latency claims.
