# m2-pipewire-probe Capability Limit

Historical: this file records the PipeWire/XDPH capability limit from the completed node and must not be read as an
active blocker for `m2-pipewire-probe`.

Named remote output capture through PipeWire/XDPH was not proven.

Evidence:

- `evidence/m2-pipewire-probe/portal-introspection.txt` shows the public and Hyprland implementation ScreenCast APIs
  expose source classes and session methods, but no named-output selector.
- `evidence/m2-pipewire-probe/portal-screencast-properties.txt` records ScreenCast versions and capability bitmasks.
- `evidence/m2-pipewire-probe/pipewire-capture-node-query.txt` returns `[]` for Video/screencast/portal/Hyprland/madobe
  nodes before and while the named Hyprland output existed.
- `evidence/m2-pipewire-probe/hyprland-monitors-with-output.json` proves the temporary named output existed.
- `evidence/m2-pipewire-probe/hyprland-monitors-after-remove.json` proves cleanup removed it.

Unblock path:

1. Run a user-consented XDPH ScreenCast session and prove the resulting PipeWire stream maps to the madobe-named
   Hyprland output, including formats/modifiers/sync metadata.
2. Or implement/probe a direct named-output capture path that bypasses portal source selection while preserving the
   security and buffer-metadata requirements.
