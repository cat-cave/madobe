# m2-video-path-validation direct-DMA-BUF-import non-claim audit

Node: `m2-video-path-validation-does-not-prove-direct-dma-buf-import-in`

Date: July 3, 2026

## Conclusion

Status: passed.

The M2 validation bundle now has an explicit, machine-checkable boundary for the encode sample. The current path is
`grim` materialized capture -> `captured-sample.png` -> `captured-sample.rgb` -> ffmpeg/NVENC process boundary with
`format=nv12`. It is not direct capture DMA-BUF import into NVENC, zero-copy capture-to-encode, or no-readback
capture-to-encode evidence.

## Files and strings reviewed

- `evidence/m2-video-path-validation/bundle-manifest.json`: adds `evidenceBoundary.machineCheckableClaims`, where
  `directCaptureDmaBufImportIntoNvencProven`, `zeroCopyCaptureToEncodeProven`, and
  `noCpuReadbackAfterGrimCaptureProven` are all `false`.
- `evidence/m2-video-path-validation/bundle-manifest.json`: records the current M2 sample path as grim materialized
  capture, PNG/RGB diagnostic artifacts, ffmpeg/NVENC process boundary, and `format=nv12`.
- `evidence/m2-video-path-validation/bundle-manifest.json`: records future direct DMA-BUF proof as future work, not
  evidence present in this bundle.
- `evidence/m2-video-path-validation/notes.md`: spells out the same boundary in prose for human readers.
- `evidence/m2-nvenc-encode-sample/notes.md`: clarifies that the NVENC proof starts after diagnostic artifacts are
  materialized and that the path is not direct capture DMA-BUF import into NVENC.

## Wording patched

Patched only allowed evidence/docs files. No code, topology, roadmap state, qd completion state, push, or PR action was
changed.

The important machine-checkable fields are:

```json
{
  "directCaptureDmaBufImportIntoNvencProven": false,
  "zeroCopyCaptureToEncodeProven": false,
  "noCpuReadbackAfterGrimCaptureProven": false,
  "nvencEncoderInvocationProven": true,
  "decoderConsumableAv1IvfSampleProven": true
}
```

## Commands run

```text
jq -e .
  evidence/m2-video-path-validation/bundle-manifest.json
  reports/qd/m2-video-path-validation-does-not-prove-direct-dma-buf-import-in/completion.json
```

Result: passed. JSON parsed successfully.

```text
jq -e '
  .evidenceBoundary.currentM2SamplePath == [
    "grim materialized capture",
    "captured-sample.png diagnostic artifact",
    "captured-sample.rgb raw diagnostic artifact",
    "ffmpeg/NVENC process boundary",
    "format=nv12 filter before av1_nvenc"
  ]
  and .evidenceBoundary.machineCheckableClaims.directCaptureDmaBufImportIntoNvencProven == false
  and .evidenceBoundary.machineCheckableClaims.zeroCopyCaptureToEncodeProven == false
  and .evidenceBoundary.machineCheckableClaims.noCpuReadbackAfterGrimCaptureProven == false
  and .evidenceBoundary.machineCheckableClaims.nvencEncoderInvocationProven == true
  and .evidenceBoundary.machineCheckableClaims.decoderConsumableAv1IvfSampleProven == true
' evidence/m2-video-path-validation/bundle-manifest.json
```

Result: passed. The manifest boundary is machine-checkable.

```text
rg -n "direct capture DMA-BUF import into NVENC|zero-copy Linux capture-to-encode|format=nv12|captured-sample\.png|captured-sample\.rgb|future work"
  evidence/m2-video-path-validation/bundle-manifest.json
  evidence/m2-video-path-validation/notes.md
  evidence/m2-nvenc-encode-sample/notes.md
  evidence/m2-video-path-validation-does-not-prove-direct-dma-buf-import-in/report.md
```

Result: passed. The explicit boundary and future-work wording are present in all reviewed docs.

```text
git diff --check
```

Result: passed. No whitespace errors were reported.

```text
just lint-lines
```

Result: passed. No file exceeded the repository line-count limit.

```text
qd gate m2-video-path-validation-does-not-prove-direct-dma-buf-import-in --phase ci
```

Result: passed. Output recorded in `qd-gate-ci.json`.

```text
nix develop -c just verify
```

Result: passed. Output recorded in `just-verify.log`.
