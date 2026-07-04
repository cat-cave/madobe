# m4-ci-job-timeouts Notes

## Scope

This node updates hosted GitHub Actions job metadata only. It does not change
product runtime code, capture, encode, transport, decode, render,
presentation, cross-device behavior, or latency behavior.

## Change

The hosted CI and nightly jobs now declare explicit conservative job timeouts:

- `.github/workflows/ci.yml` `linux`: `timeout-minutes: 45`
- `.github/workflows/ci.yml` `macos`: `timeout-minutes: 30`
- `.github/workflows/nightly.yml` `deep-checks`: `timeout-minutes: 180`

Existing commands and action versions are unchanged.

## Rationale

Explicit job-level timeouts make hung setup, dependency, security, coverage, or
mutation work fail predictably instead of relying on GitHub Actions platform
defaults.
