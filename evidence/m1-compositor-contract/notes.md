# m1-compositor-contract Notes

Implemented a new `madobe-compositor` Rust crate with a platform-neutral adapter contract.

The public API includes:

- Typed IDs: `OutputId`, `WorkspaceId`, and `SessionId`.
- Typed output configuration: dimensions, refresh rate, scale, position, color depth, and mode.
- Lifecycle/status types for outputs, bindings, desired reconcile state, and reconcile reports.
- `CompositorAdapter` trait operations for create, configure, park, remove, list, reconcile,
  and bind.
- Typed error context through `CompositorError`, `Operation`, `ErrorKind`, `Capability`,
  `Resource`, and `ConfigError`.

Validation:

- `nix develop -c just check` passed.
- `nix develop -c just test` passed.
- `rg -n "Hyprland|hypr|hyprctl|wlroots" crates apps` returned no matches.

Lifecycle coverage is in `crates/compositor/tests/lifecycle.rs` using a mock adapter. It exercises
create, configure, bind, list, park, remove, and reconcile without requiring a real compositor.

No qd graph topology was edited and the node was not marked complete.
