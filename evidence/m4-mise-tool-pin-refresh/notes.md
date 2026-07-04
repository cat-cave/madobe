# m4-mise-tool-pin-refresh Notes

## Scope

This node replaces floating native macOS mise tool selectors with explicit
reviewed versions:

- `rust` changed from `stable` to `1.96.0`.
- `just` changed from `latest` to `1.55.1`.
- `tuist` remains pinned to `4.201.0`.

The current latest values were checked with `mise latest` through the Nix dev
shell. Linux/Nix tool declarations were not edited.

## No Product Runtime Claim

Validation for this node is limited to tool pin lookup and repository checks.
This evidence does not claim any product runtime, capture, encode, transport,
decode, render, presentation, cross-device, or latency behavior was validated.
