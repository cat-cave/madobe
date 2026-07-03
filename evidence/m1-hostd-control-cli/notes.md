# m1-hostd-control-cli Evidence Notes

## Summary

Added a hostd display control layer over the platform-neutral `CompositorAdapter` contract and exposed it through `madobectl display` commands:

- `madobectl display status`
- `madobectl display create [--id <madobe-output-id>]`
- `madobectl display park [--id <madobe-output-id>]`
- `madobectl display remove [--id <madobe-output-id>]`
- `madobectl display smoke [--id <madobe-output-id>]`

The default lifecycle id is deterministic: `madobe-cli-smoke`. The default smoke mode is `1280x720@60000mhz`, scale `1/1`, position `50000x50000`, 8-bit color.

## Adapter Boundary

`hostd::run_display_action` accepts `&mut impl CompositorAdapter` and performs status, create, park, remove, and smoke actions through that trait. The only Hyprland-specific hostd function is `hostd::run_hyprland_display_action`, which constructs `HyprlandAdapter<HyprctlExecutor>` and then delegates back to the generic adapter runner. The CLI calls hostd rather than importing Hyprland internals.

## Deterministic Output

Status output sorts displays by `OutputId` and renders fields in fixed order:

```text
display status count=<n>
output id=<id> state=<ready|bound|parked> size=<width>x<height> refresh_millihertz=<mhz> scale=<num>/<den> position=<x>x<y> color_depth=<bits> workspace=<workspace|->
```

Lifecycle commands use similarly fixed one-line output. Unit tests cover explicit ids, default ids, status output, and lifecycle smoke operation ordering.

## Linux Hardware Validation

Documented safe validation commands in `docs/LINUX_ORCHESTRATOR.md` under "Remote Display Control Smoke". The runbook uses node-specific `madobe-*` output ids, captures `hyprctl -j monitors` after each lifecycle operation, and includes cleanup instructions for failed midway smoke probes.

I ran a non-mutating live `display status` probe, but this shell could not access Hyprland state and returned `compositor List failed: Unavailable { reason: "" }`. No create, park, remove, or smoke command was run against live hardware in this node.

## Validation

- `nix develop -c cargo test -p hostd -p madobectl`: passed.
- `nix develop -c just check`: passed after resolving clippy findings.
- `nix develop -c just test`: passed with 31 nextest tests and workspace doc tests; Apple tests skipped outside macOS.
- Linux orchestrator reran `nix develop -c just verify`: passed.
- Linux orchestrator ran `nix flake check`: passed for x86_64-linux.
- Linux orchestrator reran non-mutating live `display status`: failed with the same Hyprland state access error; no lifecycle mutation was attempted.
