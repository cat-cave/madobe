# m2-video-path-validation Notes

This bundle validates stable references for downstream decoder work without duplicating the upstream capture and encode artifacts.

## Included Evidence

- Capture metadata is referenced from `evidence/m2-capture-one-frame/direct-capture-frame.json` and `evidence/m2-capture-one-frame/validation-summary.json`.
- The decoder-consumable sample is `evidence/m2-nvenc-encode-sample/sample-av1.ivf`.
- Hashes and sizes for the referenced metadata, sample, logs, notes, and qd reports are recorded in `bundle-manifest.json`.
- Upstream command logs remain in their node evidence directories and this node's validation commands are recorded in `commands.log`.

## Sample For Downstream Decode

`sample-av1.ivf` is a one-frame AV1 Main IVF artifact. Local `ffprobe` validates it as:

- container: IVF / On2 IVF
- codec: AV1, tag `AV01`, mime codec string `av01.0.00M.08`
- dimensions: 160x90
- pixel format: yuv420p
- frame rate/time base: 60/1 and 1/60
- duration: 0.016667 seconds
- frames: 1
- size: 84 bytes
- SHA-256: `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`

This makes the artifact stable for a later non-Linux decoder consumer, but it does not prove that any specific non-Linux decode path has consumed it yet.

The same `ffprobe` metadata records `pixelFormat=yuv420p`, `colorRange=tv`, and unknown `colorSpace`,
`colorTransfer`, and `colorPrimaries`. The sample is therefore not HDR or color-management evidence.

## Explicit Non-Claims

The current M2 encode sample path is:

1. `grim` materialized capture.
2. `captured-sample.png` diagnostic artifact.
3. `captured-sample.rgb` raw diagnostic artifact.
4. ffmpeg/NVENC process boundary with a `format=nv12` filter before `av1_nvenc`.

That path proves that the resulting tiny AV1/IVF sample is decoder-consumable evidence. It is not evidence for direct
capture DMA-BUF import into NVENC, zero-copy Linux capture-to-encode, or no-readback Linux capture-to-encode.

This bundle does not prove:

- end-to-end streaming latency
- direct capture DMA-BUF import into NVENC
- zero-copy Linux capture-to-encode
- VideoToolbox decode
- Metal render
- HDR handling
- color-management behavior
- wide-gamut preservation
- end-to-end color accuracy
- no CPU readback after the grim-based encode sample

The upstream NVENC sample uses a grim PNG/RGB capture path and an ffmpeg/NVENC process boundary. The low-latency encoder tune is recorded only as a setting; no latency or throughput result is claimed.

## Streaming Performance Boundary

The M2 bundle contains a one-frame AV1/IVF artifact and command logs only. It is decoder-consumable sample evidence,
not live-streaming performance evidence.

The bundle does not measure or validate:

- end-to-end streaming latency
- encode latency
- throughput
- frame pacing or jitter
- CPU utilization
- GPU utilization
- network behavior
- cross-device latency

`bundle-manifest.json` records these as machine-checkable false claims under
`evidenceBoundary.machineCheckableClaims`. Downstream consumers must not treat the M2 sample, command logs, or
`av1_nvenc` low-latency tune setting as performance evidence.

A future direct DMA-BUF proof must import a capture DMA-BUF into NVENC or NVIDIA SDK/driver import APIs and record the
import and synchronization evidence. That proof is future work and is not present in this bundle.

A future HDR/color proof must use appropriate source material, tagged color metadata, and validation for HDR handling,
color-management behavior, wide-gamut preservation, and end-to-end color accuracy. That proof is future work and is not
present in this bundle.

Future streaming performance validation must run an end-to-end stream with instrumentation for streaming latency,
encode latency, throughput, frame pacing/jitter, CPU utilization, GPU utilization, network behavior, and cross-device
latency as applicable. That validation is future work and is not present in this bundle.

## Proposed qd Findings

The completion report lists exact proposed qd findings for orchestrator review. They cover the known gaps around direct DMA-BUF import, Mac decode/render, streaming performance, HDR/color behavior, and no-CPU-readback after the grim sample.
