# m2-pipewire-probe Notes

Captured on the reference Linux host on July 3, 2026.

## Versions

- PipeWire: 1.6.5, from `pipewire --version`, `pw-cli --version`, and `pw-dump --version`.
- WirePlumber: 0.5.14.
- xdg-desktop-portal: 1.20.4, from the active service process path.
- xdg-desktop-portal-hyprland: 1.3.12, from the active service process path.
- Hyprland: 0.55.2.

See `host-env.txt` and `user-services-and-journal.txt`.

## Portal Capability Result

The public portal ScreenCast interface is available at version 5. The Hyprland implementation ScreenCast interface is
available at version 3. Both report:

- `AvailableSourceTypes = 7`
- `AvailableCursorModes = 3`

The implementation service is `org.freedesktop.impl.portal.desktop.hyprland`.

XDPH logs show PipeWire connection and compositor protocol support for `zwlr_screencopy_manager_v1`,
`ext_output_image_capture_source_manager_v1`, `ext_foreign_toplevel_image_capture_source_manager_v1`,
`ext_image_copy_capture_manager_v1`, `wp_color_manager_v1`, and `wp_linux_drm_syncobj_manager_v1`.

## PipeWire Graph Result

`pw-dump` succeeded before creating a temporary output and while `madobe-qd-m2-pipewire-probe` existed. The checked-in
`pipewire-dump-*` artifacts are sanitized node-format summaries derived from those raw dumps, not raw local session
dumps.

The graph contained audio, MIDI, driver, portal, WirePlumber, and Sunshine-related nodes. `wpctl status` reported no
Video devices, sinks, sources, filters, or streams. The focused query for Video, screencast, portal, Hyprland, or
madobe-named PipeWire nodes returned `[]` before and while the temporary output existed.

This means the named Hyprland output does not appear as a pre-existing PipeWire capture node with enumerable video
formats. PipeWire video formats are not available until a ScreenCast session creates a stream.

## Named Output Probe

Created temporary output `madobe-qd-m2-pipewire-probe`, configured it to `1280x720@60` at `50000x50000`, dumped
Hyprland monitor state and PipeWire state, then removed the output. Final Hyprland state confirms the named output is
absent.

Physical monitor make/model/serial/description fields, local runtime identifiers, hostnames, process ids, machine ids,
application names, and local audio device names were redacted.

## Capability Limit

Named remote output capture through XDPH is blocked by current evidence:

- The portal API exposes source classes and session methods, not a stable noninteractive selector for
  `madobe-qd-m2-pipewire-probe`.
- No PipeWire video node or format appeared for the named output before a portal ScreenCast session.
- Starting a real ScreenCast session would require user-mediated portal selection/consent in this environment; this
  node did not claim capture success without that evidence.

Downstream one-frame capture should either collect an interactive-consent artifact proving the selected portal stream
maps to the madobe-named output, or use a direct capture path that can target the output by name with equivalent
format/modifier/sync evidence.
