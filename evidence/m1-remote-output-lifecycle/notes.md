# m1-remote-output-lifecycle Evidence Notes

## Summary

Implemented `HyprlandAdapter<E>` for the `madobe-compositor` adapter contract using the existing `madobe-hyprland` command boundary. The adapter creates named `madobe-*` headless outputs, configures them through Hyprland 0.55 Lua `eval` (`hl.monitor(...)`), parks them at a low-resolution off-session coordinate, removes them idempotently, and reconciles stale outputs by parking them.

## Live Host Evidence

Host evidence was collected against Hyprland 0.55.2 using output `madobe-qd-m1-remote-output-lifecycle`.
Physical monitor make/model/serial fields and the local Hyprland instance signature were redacted before publishing the evidence.

- `hyprland-monitors-before.json`: two developer-session monitors (`DP-2`, `DP-3`) before lifecycle work.
- `hyprland-monitors-after-create.json`: named output exists at `1280x720@60`, `50000x50000`, scale `1`.
- `hyprland-monitors-after-park.json`: same named output is parked at `640x480@30`, `50000x50000`, scale `1`.
- `hyprland-monitors-after-remove.json`: named output is absent; developer monitors remain.
- `commands.log`: exact live commands and statuses.
- `host-env.txt`: Hyprland version and host details.

Hyprland 0.55.2 reports `hyprctl keyword monitor ...` as deprecated/no-op with the current Lua config parser, so the adapter uses `hyprctl eval "hl.monitor(...)"` and verifies monitor JSON after applying configuration.

## Validation

- `nix develop -c cargo test -p madobe-hyprland`: passed.
- `nix develop -c just check`: passed.
- `nix develop -c just test`: passed; Apple tests skipped because this is Linux.

## Cleanup

The evidence script removed `madobe-qd-m1-remote-output-lifecycle` and then ran a final cleanup remove. The final cleanup reported `output not found`, confirming the named output was already absent.
