# m2-capture-stream-proof Blocker

`m2-capture-stream-proof` improved the evidence but does not unblock `m2-capture-one-frame`.

Historical: this file records the capture-stream blocker context from the completed node and must not be read as an
active blocker for `m2-capture-stream-proof`.

The reference Linux host can create and clean up a node-scoped Hyprland headless output named
`madobe-qd-m2-capture-stream-proof`, and a direct wlroots screenshot client can target that output by name when
launched with the live Wayland socket:

- `evidence/m2-capture-stream-proof/direct-grim-named-output-with-wayland-metadata.txt`
- `evidence/m2-capture-stream-proof/validation-summary.json`

The portal path is still blocked. `CreateSession` returned a request path, but the one-off bus caller did not own the
request response lifecycle. The attempted follow-up `SelectSources` call failed with `Access denied`, and PipeWire
dumps after the portal error showed no video, screencast, portal, Hyprland, or madobe-named stream node:

- `evidence/m2-capture-stream-proof/portal-create-session.txt`
- `evidence/m2-capture-stream-proof/portal-select-sources.err`
- `evidence/m2-capture-stream-proof/pipewire-focused-after-portal-error-summary.json`

The direct `grim` proof is useful but not sufficient for `m2-capture-one-frame` because it yields a PNG artifact rather
than the required frame metadata: format, modifier, timestamp, and sync mode.

To unblock `m2-capture-one-frame`, implement or run a client that owns the portal request lifecycle, receives the
`org.freedesktop.portal.Request.Response`, prompts for user consent, opens the returned PipeWire remote, and persists
the resulting stream metadata. An equivalent direct capture API is also acceptable if it exposes the same metadata.
