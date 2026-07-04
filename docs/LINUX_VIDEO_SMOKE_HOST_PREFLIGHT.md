# Linux Video Smoke Host Preflight

This is the Linux host-side preflight for `m3-cross-device-video-smoke`.
It records the command, host environment, network state, sample hash, and
start/stop evidence available before Mac client coordination.

## Host Command

Run host-side display lifecycle commands from Nix:

```sh
HOST_OUTPUT=madobe-qd-m3-linux-video-smoke-host-preflight
nix develop -c cargo run -q -p madobectl -- display status
nix develop -c cargo run -q -p madobectl -- display create --id "$HOST_OUTPUT"
nix develop -c cargo run -q -p madobectl -- display remove --id "$HOST_OUTPUT"
```

For a cleanup-safe single command when a live Hyprland session is available:

```sh
HOST_OUTPUT=madobe-qd-m3-linux-video-smoke-host-preflight
nix develop -c cargo run -q -p madobectl -- display smoke --id "$HOST_OUTPUT"
```

If any create/smoke command fails midway, run the matching remove command
with the same output id and capture the final `hyprctl -j monitors` state.

## Recorded Evidence

Evidence for this preflight is under:

```text
evidence/m3-linux-video-smoke-host-preflight/
```

The captured host evidence includes:

- `host-env.txt`: worktree, branch, commit, OS, Nix, and session variables.
- `gpu-driver.txt`: NVIDIA driver/GPU probe and kernel module state.
- `hyprland-availability.txt`: Hyprland command/socket availability.
- `pipewire-availability.txt`: PipeWire and WirePlumber service state.
- `network-firewall.txt`: host IPs, routes, listening ports, firewall service
  status, and available rule inspection.
- `sample-hash.txt`: AV1 IVF sample source, hash, size, and ffprobe metadata.
- `nix-command-validation.log`: Nix-wrapped command validation.
- `start-stop.log`: host start/stop command attempt and exit codes.

## Scope Exclusions

This preflight is Linux-only. It explicitly does not prove Mac decode,
VideoToolbox behavior, Metal render, Mac presentation/display, end-to-end
latency, frame pacing, cross-device render correctness, or user-visible Mac
presentation. Those claims require later Mac-side evidence.
