# m4 Compositor Identity Mutation Hardening Notes

## Baseline

The qd node supplied this focused workspace mutation baseline cluster from
main:

- `crates/compositor/src/ids.rs`: `OutputId::as_str` survived replacement
  with empty and fixed strings.
- `crates/compositor/src/ids.rs`: `WorkspaceId::as_str` survived replacement
  with empty and fixed strings.
- `crates/compositor/src/ids.rs`: `SessionId::as_str` survived replacement
  with empty and fixed strings.
- `crates/compositor/src/status.rs`: `OutputStatus::workspace` survived
  replacement with `None`.

## Changes

Added focused compositor unit tests in:

- `crates/compositor/src/ids.rs`
- `crates/compositor/src/status.rs`

The ID tests construct valid `OutputId`, `WorkspaceId`, and `SessionId` values
with distinct strings and assert that `as_str()` returns the exact original
identifier. This catches empty-string and fixed-string accessor substitutions.

The status tests construct `OutputStatus` values with both a bound workspace
and no workspace. They assert that `workspace()` returns `Some(&WorkspaceId)`
for the bound state and `None` for the unbound state.

## Verification

Final verification passed:

```text
nix develop -c cargo test -p madobe-compositor --all-features
nix develop -c just check
```

## Residual Risk

No targeted cargo-mutants rerun was performed for this node. The work is
limited to focused regression tests for the baseline survivor cluster and does
not change compositor runtime behavior.
