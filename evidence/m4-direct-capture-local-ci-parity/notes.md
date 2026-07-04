# m4-direct-capture-local-ci-parity Notes

## Summary

Updated the Linux local CI parity gate so `nix develop -c just ci-local`
invokes `direct-capture-preflight` before broader verification and coverage
checks. PR and repository guidance now identify the direct-capture preflight as
part of Linux PR readiness while keeping macOS validation on the native
`just macos-check` path.

## Boundary

This node only changes local verification command ordering and documentation.
It does not make runtime capture, product QUIC, portal, capture-to-NVENC,
zero-copy, cross-device, decode, render, presentation, or latency claims.
