# m4-macos-ci-jq-tooling Notes

## Scope

This node updates hosted macOS GitHub Actions setup only. It does not change
product runtime code, capture, encode, transport, decode, render,
presentation, cross-device behavior, or latency behavior.

## Change

The hosted macOS CI job now installs `jq` explicitly in the same Homebrew step
as the existing Swift tooling:

```sh
brew install jq swiftformat swiftlint
```

The step still runs after the hosted-runner `aws/tap` cleanup and before
`just macos-check`. Linux CI and Nix tool declarations are unchanged.

## Rationale

The shared `just check` path now runs `just qd-reports-check`, and that report
validator requires `jq`. Hosted macOS CI should not rely on `jq` being present
in the runner image.
