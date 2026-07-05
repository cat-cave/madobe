# m2-portal-screencast-client-proof Blocker

Historical: this file records the portal screencast client blocker context from the completed node and must not be read
as an active blocker for `m2-portal-screencast-client-proof`.

The portal client proof narrows, but does not clear, the M2 capture blocker.

The patched GLib/GDBus helper successfully owned the portal request lifecycle for `CreateSession`, received a response,
and extracted the returned `session_handle`:

- `evidence/m2-portal-screencast-client-proof/rerun5-portal-client.txt`

The helper then called `SelectSources`, but no response arrived within the bounded 30-second wait. Because
`SelectSources` did not complete, the helper never reached `Start`, never received stream ids, and never called
`OpenPipeWireRemote`.

PipeWire state after the helper showed no video, screencast, portal, Hyprland, or madobe-named stream node:

- `evidence/m2-portal-screencast-client-proof/rerun5-pipewire-after-summary.json`

The node-scoped Hyprland output was removed after the probe:

- `evidence/m2-portal-screencast-client-proof/rerun5-hyprland-monitors-after-remove.json`

`m2-capture-one-frame` must remain blocked until a consented portal flow returns stream ids and PipeWire metadata, or an
equivalent direct capture API exposes format, modifier, timestamp, and sync metadata.
