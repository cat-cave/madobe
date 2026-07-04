# m3-linux-video-smoke-host-preflight Notes

Captured on the Linux host in the qd worktree
`/home/trevor/projects/madobe/.qd/worktrees/m3-linux-video-smoke-host-preflight`.

The documented host command is runnable from Nix:

```sh
HOST_OUTPUT=madobe-qd-m3-linux-video-smoke-host-preflight
nix develop -c cargo run -q -p madobectl -- display status
nix develop -c cargo run -q -p madobectl -- display create --id "$HOST_OUTPUT"
nix develop -c cargo run -q -p madobectl -- display remove --id "$HOST_OUTPUT"
```

`nix-command-validation.log` records that `madobectl hello` succeeds from the
Nix shell. The display command path is also runnable from Nix, but in this
captured shell it returned compositor unavailable because
`HYPRLAND_INSTANCE_SIGNATURE` was unset.

`start-stop.log` records the host-side start/stop attempt using the node-scoped
output id `madobe-qd-m3-linux-video-smoke-host-preflight`. The create and remove
commands did not reach the live Hyprland instance in this shell. That is
recorded as host availability evidence, not as a successful live display
lifecycle claim.

`hyprland-availability.txt` records that `hyprctl` is installed and Hyprland
runtime sockets are present under `/run/user/1000/hypr`, but the current shell
does not have `HYPRLAND_INSTANCE_SIGNATURE`. `pipewire-availability.txt` records
active PipeWire, PipeWire PulseAudio, and WirePlumber user services.

`network-firewall.txt` records host networking. The primary LAN address observed
from the default route is `192.168.1.23`; the Tailscale address is
`100.120.148.20`. `nftables`, `firewalld`, and `iptables` system services were
reported inactive or missing, and `nft`/`iptables` rule inspection commands were
unavailable in the captured environment.

The expected sample source for later cross-device video smoke is
`evidence/m2-nvenc-encode-sample/sample-av1.ivf`. `sample-hash.txt` records
SHA-256
`51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`, size
84 bytes, IVF container, AV1 codec, 160x90 dimensions, and one frame.

This evidence explicitly excludes Mac decode, VideoToolbox decode, Metal render,
Mac presentation/display, cross-device render correctness, frame pacing, and
latency claims.
