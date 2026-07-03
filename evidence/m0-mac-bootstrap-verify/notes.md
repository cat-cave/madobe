# m0-mac-bootstrap-verify Notes

The MacBook checkout is live against `https://github.com/cat-cave/madobe.git` and current with main commit
`a2bd90f9f26c378112282f597d5eae48afdc2582` at the time of verification.

The qd method gate is acknowledged as `trevor-mac-orchestrator`. The worktree qd cache was initialized with
`qd migrate` and synced from `roadmap/qd-export.json`.

macOS verification uses native tools, not Nix. This follows `docs/MACOS_ORCHESTRATOR.md` and the current CI split:
Linux CI uses Nix, while macOS uses Mise, Homebrew SwiftFormat/SwiftLint, Tuist, and Xcode.

Validation passed:

- `just macos-bootstrap`
- `just macos-check`
- `tuist generate`
- `xcodebuild test -scheme MadobeMac -destination 'platform=macOS'`

The current Xcode evidence contains a non-blocking warning about multiple matching macOS destinations. A prior
Mac startup also emitted a CoreSimulator out-of-date warning while still passing the native macOS destination test;
that warning is preserved in `core-simulator-warning.txt` as non-blocking evidence.

PR #3 CI follow-up found a Linux `typos` false positive in the raw Xcode log. Xcode emits the Swift compiler flag
`-disable-cmo`; `typos` suggested `com`. The branch now allows `cmo` in `typos.toml` so raw command evidence can be
kept intact.
