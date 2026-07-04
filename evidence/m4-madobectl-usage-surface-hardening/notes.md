# m4-madobectl-usage-surface-hardening Notes

## Scope

- Added the dispatched `madobectl video-smoke` send and receive entry points to top-level usage text.
- Added library-level coverage proving top-level usage includes the video-smoke entry points.
- Added transport CLI usage-error coverage for unknown video-smoke commands and missing required flags.
- Left executable dispatch semantics unchanged.

## Boundary

This node only changes CLI usage text and parser/usage tests.
It makes no product runtime, capture, encode, transport, decode, render,
presentation, cross-device, or latency claim.

## Validation

Recorded in `evidence/m4-madobectl-usage-surface-hardening/commands.log`:

- `nix develop -c cargo test -p madobectl`
- `nix develop -c cargo test -p madobe-transport video_smoke`
- `nix develop -c just check`
- `git diff --check`
