# m2-portal-chooser-interaction-proof Blocker

Historical: this file records the portal chooser blocker context from the completed node and must not be read as an
active blocker for `m2-portal-chooser-interaction-proof`.

The chooser interaction proof confirms that the portal UI appears, but it does not clear the M2 capture blocker.

The helper successfully received the `CreateSession` response and extracted a `session_handle`. It then called
`SelectSources`, and the xdg-desktop-portal chooser appeared with the generated output listed as a selectable screen.

Evidence:

- `evidence/m2-portal-chooser-interaction-proof/portal-client.txt`
- `evidence/m2-portal-chooser-interaction-proof/clients-at-chooser-detection.json`
- `evidence/m2-portal-chooser-interaction-proof/kbd3-portal-client.txt`
- `evidence/m2-portal-chooser-interaction-proof/kbd3-clients-at-chooser-detection.json`

A targeted keyboard interaction was sent to the visible chooser, but `SelectSources` still did not return within the
bounded 70-second wait. Because `SelectSources` did not complete, the helper never reached `Start`, never received
stream ids, and never called `OpenPipeWireRemote`.

Click automation was not available in this session: `ydotool` was installed but could not access its daemon socket.

PipeWire state after the probe showed no video, screencast, portal, Hyprland, or madobe-named stream node:

- `evidence/m2-portal-chooser-interaction-proof/pipewire-after-summary.json`

Cleanup evidence shows no leftover generated output or chooser:

- `evidence/m2-portal-chooser-interaction-proof/kbd3-hyprland-monitors-after-remove.json`
- `evidence/m2-portal-chooser-interaction-proof/kbd3-clients-after-cleanup.json`

`m2-capture-one-frame` must remain blocked until a consented portal flow returns stream ids and PipeWire metadata, or an
equivalent direct capture API exposes format, modifier, timestamp, and sync metadata.
