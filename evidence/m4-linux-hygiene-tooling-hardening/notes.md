# m4 Linux Hygiene Tooling Hardening Notes

## Boundary

Linux `nix develop -c just check` now treats the repo hygiene tools as required:
Taplo for TOML formatting, shfmt for shell formatting, typos for spelling,
markdownlint-cli2 for Markdown, actionlint for GitHub Actions workflows, and the
existing line-limit check.

On Linux, the shared `fmt` and `check` recipes fail through `just require-tool`
before running each required hygiene tool. Missing Linux tools no longer
silently skip checks.

On macOS, Taplo, shfmt, typos, markdownlint-cli2, and actionlint run when
installed and otherwise print explicit non-Linux skip messages. SwiftFormat and
SwiftLint remain macOS-native. They are not part of Linux `just check`;
`just macos-check` runs the shared checks first and then runs
`macos-swiftformat`, `macos-swiftlint`, Tuist generation, and Xcode tests on
Darwin. Outside Darwin, the macOS Swift recipes print skip messages, matching
the existing non-macOS behavior for Apple-specific validation.

## Product Claims

This node only changes repo hygiene tooling and documentation. It does not
change product QUIC, decode, render, presentation, or latency claims.
