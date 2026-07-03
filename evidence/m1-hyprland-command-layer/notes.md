# m1-hyprland-command-layer Notes

Implemented `madobe-hyprland` as the focused Hyprland command and event layer.

- `CommandExecutor` is the injectable boundary for tests and future host code.
- `HyprctlExecutor` is the only product code that invokes `std::process::Command`.
- `HyprlandClient` parses `hyprctl -j monitors`, `workspaces`, and `activeworkspace`.
- Hyprland JSON parsing is dependency-free and focused on the captured `hyprctl` shapes.
- Parser tests use the sanitized fixtures from `fixtures/hyprland/`.
- Errors are typed and carry `CommandContext` with program and arguments.
- Event support includes a socket2 line parser and a `BufRead` event source.

Validation passed:

- `nix develop -c just check`
- `nix develop -c just test`
- `nix develop -c just security`

Residual risks:

- This node does not exercise a live Hyprland socket or live `hyprctl`; that belongs to the later hardware lifecycle validation node.
- The event source is line-oriented and fixture/mock ready, but no socket transport is opened yet.
- The internal JSON parser intentionally supports the JSON types needed by current Hyprland fixtures rather than a broad general-purpose JSON API.
