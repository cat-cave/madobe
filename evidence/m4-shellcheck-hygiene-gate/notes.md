# m4-shellcheck-hygiene-gate Notes

## Scope

- Added ShellCheck to the public `just check` hygiene surface for tracked shell scripts.
- Preserved the repository's explicit Darwin-native skip behavior when ShellCheck is unavailable outside Linux/Nix.
- Fixed the ShellCheck warning in `nix/direct-capture-preflight.sh` without changing its preflight scope.
- Updated engineering documentation so the documented Linux hygiene gate includes ShellCheck.

## Boundary

This node only changes repository hygiene commands, documentation, and evidence.
It makes no product runtime, capture, encode, transport, decode, render,
presentation, cross-device, or latency claim.

## Validation

Recorded in `evidence/m4-shellcheck-hygiene-gate/commands.log`:

- `nix develop -c shellcheck nix/direct-capture-preflight.sh`
- `nix develop -c just check`
- `git diff --check`
