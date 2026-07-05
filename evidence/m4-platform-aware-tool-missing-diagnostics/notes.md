# Platform-Aware Tool Diagnostics Notes

The shared `require-tool` helper now chooses its missing-tool guidance at
runtime:

- Linux and other non-Darwin platforms continue to point users at
  `nix develop`.
- macOS points users at `just macos-bootstrap` for baseline native tooling or
  installing the missing tool on the native macOS `PATH`.

The helper line is prefixed with `@` so Just does not echo the whole shell
branch before running it. Without that, macOS users would still see the Linux
fallback text in the echoed recipe body even though the Darwin branch selected
the native message.

The coverage, mutation, and security recipes now call the same helper for their
cargo tool prerequisites, keeping missing-tool diagnostics centralized.

This node changes diagnostics only. It does not change product runtime,
capture, encode, transport, decode, render, presentation, cross-device, or
latency behavior.
