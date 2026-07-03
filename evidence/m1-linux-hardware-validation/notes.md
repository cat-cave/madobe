# m1-linux-hardware-validation Notes

Captured on the reference Linux host on July 3, 2026.

## Summary

Validated the remote display lifecycle through the real `madobectl` and Hyprland adapter path using output id
`madobe-qd-m1-linux-hardware-validation`.

The lifecycle succeeded:

- Initial `madobectl display status`: `display status count=0`.
- `display create`: ready at `1280x720@60000mhz`, scale `1/1`, position `50000x50000`.
- Second `display create` against the existing output: ready at the same configuration, exercising the adapter's
  existing-output adoption/configure path.
- `display park`: parked at `640x480@30000mhz`, scale `1/1`, position `50000x50000`.
- `display remove`: `status=removed`.
- Final `madobectl display status`: `display status count=0`.

`hyprctl -j monitors` confirms the named output existed after create/adopt/park and was absent after remove.

## Evidence

- `commands.log`: exact command sequence and exit statuses.
- `host-env.txt`: sanitized host and tool versions.
- `host.log`: selected user service journal output for the validation window; no entries were emitted.
- `validation-summary.json`: condensed lifecycle result.
- `hyprland-monitors-before.json`, `hyprland-monitors-after-create.json`,
  `hyprland-monitors-after-adopt.json`, `hyprland-monitors-after-park.json`,
  `hyprland-monitors-after-remove.json`: sanitized monitor snapshots.
- `hyprland-workspaces-before.json`, `hyprland-workspaces-after-remove.json`: sanitized workspace snapshots.
- `json-validation.log`: JSON artifact validation.
- `redaction-scan.log`: public evidence redaction scan.

## Privacy

Physical monitor make, model, serial, and description fields were redacted. Runtime paths, live Hyprland signatures,
hostnames, and process ids were redacted before commit.

## Findings

No flaky or unproven behavior was observed during this validation. No qd findings were filed for this node.
