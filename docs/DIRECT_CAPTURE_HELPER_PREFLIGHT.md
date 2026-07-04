# Direct Capture Helper Preflight

This preflight closes the M2 audit reproducibility gap for the direct Wayland
capture proof environment. It is a Linux build-environment check, not a product
capture implementation.

## Command

Run from the repo root inside the Linux Nix shell:

```sh
nix develop -c just direct-capture-preflight
```

Hosted Linux CI runs the same command after `nix flake check` and before the
repo-wide `just check`/test/security gates. The macOS CI job does not run this
preflight and remains responsible only for the native macOS check surface.
Local Linux PR readiness uses `nix develop -c just ci-local`, which invokes this
preflight before `verify` and `coverage`.

The target runs `nix/direct-capture-preflight.sh`, which:

- requires `cc`, `pkg-config`, and `wayland-scanner`;
- verifies `pkg-config` metadata for `wayland-client`, `wayland-protocols`,
  `gbm`, and `libdrm`;
- generates client protocol code for `ext-foreign-toplevel-list-v1`,
  `ext-image-capture-source-v1`, `ext-image-copy-capture-v1`, and
  `linux-dmabuf-v1` from the Nix-provided `wayland-protocols` XML files;
- compiles and links `crates/capture/tools/direct_capture_preflight.c` against
  Wayland client, GBM, and libdrm.

## Boundary

The M2 audit accepted direct capture evidence for one Linux frame from a
node-scoped Hyprland output through `ext-image-copy-capture-v1`, using a
GBM-allocated NVIDIA DMA-BUF with size, format, modifier, timestamp, damage, and
implicit/protocol-ready sync metadata. The residual risk was repo quality:
`m2-native-dmabuf-allocation-proof` used a temporary `nix-shell -p` environment,
and the repo `nix develop` shell did not expose the native Wayland/GBM build
dependencies needed to rebuild or check that proof path.

This preflight boundary is deliberately narrower than capture runtime behavior.
It proves that the repo-supported Linux dev environment exposes the build tools
and protocol metadata needed for a direct-capture helper path. It does not
connect to a compositor, create a Hyprland output, import a DMA-BUF into
Wayland, capture a frame, encode, decode, render, present, measure latency,
prove zero-copy, prove no CPU readback, or validate cross-device behavior.
