# m2-capture-stream-proof Notes

Captured on the reference Linux host on July 3, 2026.

## Result

Status: research-incomplete / blocked.

A node-scoped Hyprland headless output named `madobe-qd-m2-capture-stream-proof` was created, configured to
`1280x720@60` at `50000x50000`, and bound to disposable workspace
`madobe-qd-m2-capture-stream-proof-workspace`. Redacted Hyprland snapshots prove the output and workspace existed in
`hyprland-monitors-with-output.json` and `hyprland-workspaces-with-output.json`.

The XDPH public ScreenCast API accepted `CreateSession` and returned a request path, but the one-off `busctl` caller
did not receive a request response in the captured monitor window. Continuing with the derived session path failed at
`SelectSources` with `Call failed: Access denied`. No user-consented portal stream was produced.

Focused PipeWire dumps before and after the portal error contain no video, screencast, portal, Hyprland, or
madobe-named stream node. The only focused node match was an unrelated audio input stream.

## Direct Named-Output Checks

`grim -o madobe-qd-m2-capture-stream-proof` failed without an explicit `WAYLAND_DISPLAY` in the non-interactive
executor. A bounded retry with `XDG_RUNTIME_DIR` and `WAYLAND_DISPLAY` set explicitly succeeded and produced a PNG in
`/tmp`, recorded only as size/signature/hash in `direct-grim-named-output-with-wayland-metadata.txt`.

This proves a wlroots screenshot path can target the named Hyprland output by name when the Wayland socket is provided.
It does not prove the PipeWire/XDPH stream path and does not provide DMA-BUF format, modifier, timestamp, or sync
metadata for `m2-capture-one-frame`.

## Cleanup

The madobe-owned output was removed after the portal attempt and again after the bounded direct `grim` retry.
`hyprland-monitors-after-direct-grim-cleanup.json` contains no `madobe-qd-m2-capture-stream-proof` monitor.

## Manual Unblock

Manual portal consent is required from a client that owns the portal request lifecycle and subscribes to
`org.freedesktop.portal.Request.Response`.

Required manual action:

1. Start a ScreenCast portal session from a real client process that listens for request responses.
2. In the portal chooser, select the monitor/output corresponding to `madobe-qd-m2-capture-stream-proof`.
3. Capture the returned `streams` result, the PipeWire node id, and `OpenPipeWireRemote` fd use.
4. Dump the resulting PipeWire stream metadata and prove whether its size/source maps to the madobe output.

Do not unblock `m2-capture-one-frame` until format, modifier, timestamp, and sync metadata are available from that
stream or from an equivalent direct capture API.
