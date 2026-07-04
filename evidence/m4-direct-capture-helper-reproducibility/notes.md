# m4-direct-capture-helper-reproducibility Notes

## Result

Status: complete for implementor review.

This node adds a repo-supported direct capture build preflight:

```sh
nix develop -c just direct-capture-preflight
```

The preflight checks that the Linux dev shell exposes the native build surface
needed by the M2 direct capture proof path:

- `cc`
- `pkg-config`
- `wayland-scanner`
- `wayland-client`
- `wayland-protocols`
- `gbm`
- `libdrm`

It generates client protocol code from the Nix-provided `wayland-protocols`
XML files for:

- `ext-foreign-toplevel-list-v1`
- `ext-image-capture-source-v1`
- `ext-image-copy-capture-v1`
- `linux-dmabuf-v1`

It then compiles and links a tiny C smoke binary against Wayland client, GBM,
and libdrm. The passing run recorded:

- `wayland-client`: `1.25.0`
- `wayland-protocols`: `1.48`
- `gbm`: `26.0.3`
- `libdrm`: `2.4.133`

## M2 Residual Risk

The M2 audit accepted direct capture evidence for one Linux frame from a
node-scoped Hyprland output through `ext-image-copy-capture-v1`, using a
GBM-allocated NVIDIA DMA-BUF with size, format, modifier, timestamp, damage, and
implicit/protocol-ready sync metadata.

The residual risk was repo-quality reproducibility, not the one-frame evidence
itself. `m2-native-dmabuf-allocation-proof` used a temporary `nix-shell -p`
environment, and the repo `nix develop` shell did not expose `pkg-config`,
`wayland-scanner`, Wayland protocol metadata, GBM, and libdrm for rebuilding or
checking a direct capture helper path.

This node closes that reproducibility gap by moving those dependencies into the
repo dev shell and adding a bounded preflight target.

## Exact Boundary

This is a build/preflight path only. It proves that the repo-supported Linux
Nix environment can check the native Wayland/GBM/direct-capture helper build
surface from source.

It does not:

- connect to a compositor;
- create, configure, park, or remove a Hyprland output;
- import a DMA-BUF into Wayland;
- capture a frame;
- encode with NVENC;
- decode, render, or present on any client;
- prove zero-copy capture-to-encode;
- prove no CPU readback;
- measure latency, throughput, frame pacing, color, or HDR behavior;
- validate cross-device behavior.

Future product capture, portal, capture-to-NVENC, zero-copy, cross-device,
decode, render, presentation, and latency claims still require their own qd
nodes and hardware evidence.

## Validation

`nix develop -c just direct-capture-preflight` passed.

`nix develop -c just check` passed.

Per the implementor prompt, `qd complete`, `qd audit`, and `qd gate` were not
run; the orchestrator will run qd gate after review.
