# m4-macos-ci-brew-tap-hygiene Notes

## Warning source

Hosted `macos-15` GitHub Actions runners can include a preinstalled Homebrew tap named `aws/tap`.
When the CI job later runs `brew install swiftformat swiftlint`, Homebrew may emit a tap trust annotation for that unrelated preinstalled tap.

The warning is runner hygiene noise, not a project dependency and not local Mac setup guidance.

## Change

The macOS CI job now removes `aws/tap` before installing the native Homebrew Swift tools:

```sh
HOMEBREW_NO_REQUIRE_TAP_TRUST=1 brew untap aws/tap || true
```

The environment override is scoped only to the untap cleanup command. The
following `brew install swiftformat swiftlint` step still runs with normal
Homebrew trust checks after the unrelated hosted-runner tap has been removed.
The step is named `Remove hosted CI runner aws/tap` to keep the scope explicit.
Linux CI remains unchanged, and macOS CI continues to use native Xcode, Mise,
Homebrew, and `just macos-check`.

## Expected hosted CI validation

On a GitHub-hosted `macos-15` runner, the hygiene step should:

- remove `aws/tap` when the runner image provides it;
- do nothing when the tap is absent;
- allow `brew install swiftformat swiftlint` to run without the previous `aws/tap` tap-trust annotation.

Local validation cannot reproduce the hosted-runner preinstalled tap inventory, so hosted CI is the expected validation point for the warning disappearing.
