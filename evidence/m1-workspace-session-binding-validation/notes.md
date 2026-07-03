# M1 Workspace Session Binding Validation

Node: `m1-workspace-session-binding-validation`
Date: 2026-07-03
Platform: Linux / Hyprland 0.55.2

Validated behavior:

- Created node-scoped output `madobe-qd-m1-workspace-session-binding-validation`.
- Created disposable workspace `madobe-qd-m1-workspace-session-binding-validation-workspace`.
- Bound the disposable workspace to the madobe output through `madobectl display bind`, which exercises `HyprlandAdapter::bind_session`.
- Captured redacted Hyprland monitor/workspace JSON before creation, after output creation, after disposable workspace creation, during binding, and after cleanup.
- Removed the madobe output and confirmed the disposable workspace was absent after cleanup.

Implementation finding resolved in this node:

- Hyprland 0.55.2 rejects the old `hyprctl dispatch moveworkspacetomonitor name:<workspace> <output>` command form with a Lua parser error.
- The adapter now uses the verified Lua dispatcher object form: `hl.dispatch(hl.dsp.workspace.move({ workspace = 'name:<workspace>', monitor = '<output>' }))`.

Validation summary:

- `evidence/m1-workspace-session-binding-validation/validation-summary.json` reports `binding=passed`.
- Cleanup reports `outputAbsent=true` and `disposableWorkspaceAbsent=true`.
- `nix develop -c just verify` passed after the adapter and CLI changes.

Redaction:

- Physical monitor names, physical monitor descriptions, window addresses, window titles, local runtime paths, and the Hyprland instance signature are not committed.
- Restore commands in `commands.log` use placeholder monitor/workspace names.
