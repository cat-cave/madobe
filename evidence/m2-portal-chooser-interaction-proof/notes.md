# m2-portal-chooser-interaction-proof Notes

Captured on the reference Linux host on July 3, 2026.

## Result

Status: completed with blocker.

The portal chooser appears for a ScreenCast `SelectSources` request. The first probe created a node-scoped headless
Hyprland output and observed the chooser on the generated workspace. The final probe then tried a targeted keyboard
sequence against the visible chooser: `Tab Down Down Space Return Tab Tab Return`.

The helper still timed out waiting for `SelectSources` after the visible chooser interaction. Because `SelectSources`
did not return, the helper never reached `Start`, never received stream ids, and never called `OpenPipeWireRemote`.

`ydotool` was present but not usable from this session because it could not access its daemon socket, so click-based UI
automation was not available for this node.

## Cleanup

Both generated outputs were removed after the probes:

- `madobe-qd-m2-portal-chooser-interaction-proof`
- `madobe-qd-m2-portal-chooser-interaction-proof-kbd3`

The final live cleanup check found no remaining portal chooser window and no generated madobe output.

## Disposition

`m2-capture-one-frame` remains blocked. The next capture proof needs either a reliable way to approve the portal chooser
and receive `SelectSources`/`Start` responses, or a direct capture path that exposes format, modifier, timestamp, and
sync metadata without relying on this chooser flow.
