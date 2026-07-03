# m2-portal-manual-consent-frame-proof Notes

Captured on the reference Linux host on July 3, 2026.

## Result

Status: portal manual-consent path proven.

The final probe created and configured a node-scoped Hyprland output:

- `madobe-qd-m2-portal-manual-consent-frame-proof`
- `1280x720@60`
- parked at `50000x50000`

It also created and bound a disposable workspace:

- `madobe-qd-m2-portal-manual-consent-frame-proof-workspace`

The probe then returned focus to the original visible workspace before starting the portal helper. This kept the
xdg-desktop-portal ScreenCast chooser visible while the node output remained available as the selected source.

Consent was completed with keyboard automation through `wtype`:

```text
Tab, Down, Down, Space, Return, Tab, Tab, Return
```

`ydotool` was installed, but click automation was not available because no `ydotoold` socket existed.

## Portal Path

The GLib/GDBus helper completed the expected public portal path:

- `CreateSession` returned a session handle.
- `SelectSources` returned response code `0`.
- `Start` returned response code `0`.
- `Start` returned one stream, node id `181`.
- The returned stream properties reported `source_type=1`, `size=(1280, 720)`, and `position=(0, 0)`.
- `OpenPipeWireRemote` returned a PipeWire fd.

## PipeWire Metadata

While the helper held the PipeWire remote open, `pw-dump` showed a portal video source node with:

- `media.class=Video/Source`
- `format=BGRA`
- `size=1280x720`
- DRM modifier alternatives, including a non-linear default and the explicit linear modifier fallback
- `maxFramerate=120/1`

This proves the manual-consent portal stream path through stream id and PipeWire metadata. It does not prove a captured
frame buffer, timestamp, or sync timeline. The downstream frame node still needs to connect to the returned PipeWire
remote and capture a frame with timestamp and sync evidence.

## Cleanup

The final run removed the node output and left no portal chooser:

- `after-cleanup-hyprland-monitors.json` contains only redacted physical monitors.
- `after-cleanup-portal-clients.json` is `[]`.
- `after-cleanup-hyprland-workspaces.json` contains no node workspace.

## Disposition

`m2-capture-one-frame` can proceed past the portal-consent and stream-id blocker using this evidence. It must still
prove actual frame capture metadata before claiming size, format, modifier, timestamp, and sync as a frame result.
