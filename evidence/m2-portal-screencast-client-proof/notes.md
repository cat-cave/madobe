# m2-portal-screencast-client-proof Notes

Captured on the reference Linux host on July 3, 2026.

## Result

Status: completed with blocker.

A patched GLib/GDBus helper owned the portal `Request.Response` lifecycle for `CreateSession` and extracted the
returned `session_handle`. The helper then called `SelectSources`, but no response arrived within the bounded
30-second wait. No `Start` call, stream ids, or `OpenPipeWireRemote` fd were reached.

The observed blocker is now narrower than the previous bus probe:

- `CreateSession` succeeds and response handling works.
- `SelectSources` remains blocked before stream selection completes.
- PipeWire state after the helper contains no video, screencast, portal, Hyprland, or madobe-named stream node.

## Cleanup

The node-scoped output `madobe-qd-m2-portal-screencast-client-proof` was removed after the probe. The cleanup snapshot
contains no monitor with that name.

## Disposition

`m2-capture-one-frame` remains blocked. The next proof must either make the portal chooser/consent UI produce a
`SelectSources` response and then a `Start` response with streams, or use an equivalent direct capture API that exposes
format, modifier, timestamp, and sync metadata.
