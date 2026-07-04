# m2-nvenc-encode-sample Notes

Captured on the reference Linux host on July 3, 2026.

## Result

Status: complete.

A node-scoped Hyprland headless output named `madobe-qd-m2-nvenc-encode-sample` was created at `160x90@60` and
captured with `grim`. The captured PNG was converted to one deterministic `rgb24` raw frame and encoded through
ffmpeg's NVIDIA encoder into `sample-av1.ivf`.

The encoded artifact is intentionally tiny: 84 bytes. `ffprobe-sample-av1.txt` identifies it as one AV1 frame in an
IVF container, `sample-av1-header.txt` starts with `DKIF` and codec tag `AV01`, and `artifact-sha256.txt` records the
artifact hash.

The grim PNG and raw RGB artifacts (`captured-sample.png` and `captured-sample.rgb`) are materialized diagnostic
sample inputs for this encode proof. They do not prove no CPU readback, zero-copy capture, or direct capture-to-NVENC
DMA-BUF transfer.

## Boundary

No direct NVIDIA SDK or driver FFI was introduced. The unsafe-prone host boundary is isolated as
`madobe-encode-nv-sys`, which builds the ffmpeg process invocation. The safe product API is `madobe-encode`, which
accepts product-level settings (`DesktopAv1EncodeSettings::balanced_realtime`) and captured sample frame metadata.
Both crates keep `#![forbid(unsafe_code)]`; `unsafe-boundary-scan.log` records the focused scan.

## Encoder Settings

- Encoder: `av1_nvenc`
- Container: IVF
- Input: one `rgb24` raw frame, `160x90`, `60` fps
- Conversion: `format=nv12` before NVENC
- Preset: `p4`
- Tune: `ll`
- Rate control: `vbr`
- Constant quality: `28`
- Bitrate: `0`
- GOP: `60`

The first encode attempt intentionally preserved in evidence did not force NV12 and failed because ffmpeg selected
`YUV444P`, which this `av1_nvenc` path rejected. The boundary now forces NV12 before the encoder.

## Cleanup

The node-scoped output was removed after capture. `hyprland-monitors-after-remove.json` contains no
`madobe-qd-m2-nvenc-encode-sample` monitor.
