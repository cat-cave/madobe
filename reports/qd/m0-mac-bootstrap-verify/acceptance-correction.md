# Acceptance Correction

The original qd acceptance text for `m0-mac-bootstrap-verify` mentions:

- `nix develop -c just check`
- `nix develop -c just apple-generate`
- `nix develop -c just apple-test`

That text is stale for macOS. Current project docs and CI split supersede it:

- `docs/ORCHESTRATION.md` says the shared command surface is `just`, with platform-specific environment managers.
- `docs/MACOS_ORCHESTRATOR.md` says macOS does not use Nix and should use native Mise, Homebrew, Xcode, and Tuist.
- PR #2 updated macOS CI to avoid Nix and use native macOS tooling.

For this node, the corrected macOS acceptance mapping is:

- `just macos-bootstrap` succeeds or records an exact blocker.
- `just macos-check` succeeds or records an exact blocker.
- Tuist generation succeeds and is logged.
- `xcodebuild test -scheme MadobeMac -destination 'platform=macOS'` succeeds and is logged.

No qd topology change is needed. The node completion evidence uses the corrected macOS-native command surface.
