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

## Explicit Non-Claims

This bundle does not prove:

- end-to-end streaming latency
- direct DMA-BUF import into NVENC
- VideoToolbox decode
- Metal render
- HDR handling
- no CPU readback after the grim-based encode sample

The upstream NVENC sample uses a grim PNG/RGB capture path and an ffmpeg/NVENC process boundary. The low-latency encoder tune is recorded only as a setting; no latency or throughput result is claimed.

## Proposed qd Findings

The completion report lists exact proposed qd findings for orchestrator review. They cover the known gaps around direct DMA-BUF import, Mac decode/render, streaming performance, HDR/color behavior, and no-CPU-readback after the grim sample.
