# m1-hyprland-fixtures Notes

Captured sanitized Hyprland snapshots from the reference Linux host on July 3, 2026.

Committed fixtures:

- `fixtures/hyprland/monitors.json`
- `fixtures/hyprland/workspaces.json`
- `fixtures/hyprland/clients.json`
- `fixtures/hyprland/activeworkspace.json`

The raw `hyprctl` outputs were written only to `/tmp/m1-hyprland-fixtures-raw` and were not committed.

Sanitization performed:

- Replaced monitor `description` and `serial` values.
- Replaced workspace names with `workspace-<id>` or `special:redacted-<id>`.
- Replaced non-zero workspace `lastwindow` addresses.
- Replaced client window addresses, app classes, titles, initial classes, initial titles, grouped window addresses, and non-empty swallowing addresses.
- Set client PIDs to `0`.
- Replaced active workspace `lastwindow` and `lastwindowtitle`.
- Scanned committed fixture files for user names, home paths, runtime paths, URLs, IP addresses, and common sensitive keywords; no matches were found.

Fixture shape observations:

- `monitors.json` is an array with 2 monitors.
- `workspaces.json` is an array with 2 workspaces.
- `clients.json` is an array with 2 clients.
- `activeworkspace.json` is a single object, matching `hyprctl -j activeworkspace`.
- The Hyprland 0.55.2 monitor shape includes fields such as `colorManagementPreset`, `directScanoutBlockedBy`, `hardwareCursorsInUse`, `sdrBrightness`, `sdrMinLuminance`, `sdrMaxLuminance`, and `sdrSaturation`.
- The client shape includes newer metadata fields including `acceptsInput`, `contentType`, `overFullscreen`, `stableId`, `visible`, `xdgDescription`, and `xdgTag`.
- The captured workspace ids were `11` and `12`; parser tests should not assume low or contiguous workspace ids.

Parser follow-up needs:

- Treat workspace names, window titles, app classes, monitor descriptions, and monitor serials as user data.
- Accept unknown/new Hyprland JSON fields without failing fixture parsing.
- Do not require `activeworkspace` to be an array.
- Treat non-zero workspace `lastwindow` values as window addresses requiring redaction in committed fixtures and parser golden data.
- Add an event-stream fixture later if a deterministic, non-invasive socket2 event sample is needed. A 2 second `nc -U .../.socket2.sock` sample produced no events during this capture, so no empty event fixture was committed.
