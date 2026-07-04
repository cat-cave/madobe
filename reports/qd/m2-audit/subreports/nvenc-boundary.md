# M2 NVENC Boundary Audit

Node: `m2-audit`

Date: 2026-07-04

## Reviewed Files and Evidence

- `docs/ORCHESTRATION.md`
- `docs/LINUX_ORCHESTRATOR.md`
- `roadmap/qd-export.json` entries for `m2-nvenc-encode-sample`,
  `m2-video-path-validation`, `m2-audit`, and promoted M2 findings
- `crates/encode/src/lib.rs`
- `crates/encode-nv-sys/src/lib.rs`
- `crates/transport/src/video_smoke/mod.rs`
- `crates/transport/tests/lan_video_smoke.rs`
- `evidence/m2-nvenc-encode-sample/commands.log`
- `evidence/m2-nvenc-encode-sample/notes.md`
- `evidence/m2-nvenc-encode-sample/validation-summary.json`
- `evidence/m2-nvenc-encode-sample/unsafe-boundary-scan.log`
- `evidence/m2-nvenc-encode-sample/sample-av1.ivf`
- `evidence/m2-nvenc-encode-sample/artifact-sha256.txt`
- `evidence/m2-nvenc-encode-sample/artifact-sizes.txt`
- `evidence/m2-nvenc-encode-sample/ffprobe-sample-av1.txt`
- `evidence/m2-video-path-validation/bundle-manifest.json`
- `evidence/m2-video-path-validation/notes.md`
- `reports/qd/m2-nvenc-encode-sample/completion.json`
- `reports/qd/m2-nvenc-encode-sample/audit.json`
- `reports/qd/m2-video-path-validation/completion.json`
- `reports/qd/m2-video-path-validation/audit.json`
- Follow-up/non-claim evidence under `evidence/m2-video-path-validation-does-not-*`
- M3 consumers under `evidence/m3-lan-video-smoke-harness`,
  `evidence/m3-videotoolbox-decode-sample`, and related transport tests

`qd node show m2-audit --full` was attempted first, but it failed with:

```text
DB schema is older than this qd binary (no schema_migrations table found). Run qd migrate.
```

I did not run `qd migrate` because this audit was instructed not to modify
qd graph/state.

## Findings by Severity

### P0

None.

### P1

None.

### P2

- Encode parameter validation hardening gap. This does not block M3 fixture use,
  but the encode command builders accept unchecked zero or nonsensical encode
  parameters. `DesktopAv1EncodeSettings::balanced_realtime` stores any
  `frame_rate` in `crates/encode/src/lib.rs:30`, `CapturedSampleFrame::new`
  stores any `width`/`height` in `crates/encode/src/lib.rs:69`, and
  `NvencAv1Request::new` stores any `gop_frames`/`constant_quality` in
  `crates/encode-nv-sys/src/lib.rs:112`. `FfmpegNvenc::av1_command` then
  formats those values directly into `-s:v`, `-r`, `-g`, and `-cq` arguments in
  `crates/encode-nv-sys/src/lib.rs:176`. This is not shell injection because
  the code returns an argv vector, but it should be hardened before the API
  becomes a general runtime encode path.

### P3

- Minor historical command-log inconsistency:
  `evidence/m2-nvenc-encode-sample/commands.log:83` records a successful NV12
  retry with `-tune hq`, while the final successful update at line 94 records
  `-tune ll`. Final notes, validation summary, manifest, and current Rust
  mapping all consistently use `tune=ll`, so this is audit noise rather than a
  correctness bug.
- The sample is a tiny single-frame fixture, not representative stream content.
  This is already documented in `evidence/m2-nvenc-encode-sample/notes.md:13`,
  `evidence/m2-nvenc-encode-sample/notes.md:28`, and
  `evidence/m2-video-path-validation/notes.md:84`; no new blocker is needed.

## Acceptance Coverage Notes

- Unsafe boundary: covered. Both encode crates use `#![forbid(unsafe_code)]` in
  `crates/encode/src/lib.rs:2` and `crates/encode-nv-sys/src/lib.rs:2`.
  `evidence/m2-nvenc-encode-sample/unsafe-boundary-scan.log` reports only
  those two lines. No direct NVIDIA SDK/driver FFI is present in the reviewed
  encode crates.
- Command construction: acceptable for the current fixture path.
  `FfmpegNvenc::av1_command` builds `FfmpegCommand { program, args }` rather
  than a shell string in `crates/encode-nv-sys/src/lib.rs:182`.
  `display_command` is an escaped evidence/log rendering helper at
  `crates/encode-nv-sys/src/lib.rs:140`. Argument validation is the main
  hardening gap.
- NVENC settings evidence: covered. The current path records `rgb24` input,
  `format=nv12`, `av1_nvenc`, `preset=p4`, `tune=ll`, `vbr`, `cq=28`, GOP 60,
  and IVF output in `evidence/m2-nvenc-encode-sample/notes.md:45` and
  `evidence/m2-video-path-validation/bundle-manifest.json:95`.
- Sample hash/size evidence: covered. I independently verified
  `evidence/m2-nvenc-encode-sample/sample-av1.ivf` as 84 bytes with SHA-256
  `51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875`,
  matching `evidence/m2-video-path-validation/bundle-manifest.json:86`.
- Sample format evidence: covered. `ffprobe` reports IVF / AV1 Main / `AV01`,
  160x90, yuv420p, one frame, and a 1/60 time base. The header starts with
  `DKIF` and `AV01`.
- Low-latency claims/non-claims: covered. The repo records `tune=ll` as a
  setting only and explicitly disclaims latency/performance proof in
  `evidence/m2-nvenc-encode-sample/notes.md:28`,
  `evidence/m2-video-path-validation/notes.md:80`, and
  `evidence/m2-video-path-validation/bundle-manifest.json:126`.
- Direct DMA-BUF/no-readback/zero-copy boundary: covered. The encode sample
  path is explicitly `grim` -> PNG -> RGB -> ffmpeg/NVENC process boundary with
  `format=nv12`, not direct capture DMA-BUF import into NVENC. See
  `evidence/m2-video-path-validation/bundle-manifest.json:110` and explicit
  non-claims at `evidence/m2-video-path-validation/bundle-manifest.json:303`.
- M3 input reliability: sufficient for fixture/decode-smoke use. M3 transport
  pins the exact hash in `crates/transport/src/video_smoke/mod.rs:30` and
  validates metadata, byte count, and SHA-256 in
  `crates/transport/src/video_smoke/mod.rs:213`. The LAN smoke evidence records
  the same 84-byte hash in `evidence/m3-lan-video-smoke-harness/result.json:9`.
  Mac VideoToolbox evidence later decoded this same hash with
  `decodeStatus=decoded`, 160x90, pixel format `420v`, and hardware decode
  enabled in `evidence/m3-videotoolbox-decode-sample/decode-report.json:6`.

## Recommended Dispositions and Follow-Up Nodes

- Accept `m2-nvenc-encode-sample` and `m2-video-path-validation` for M3
  fixture/sample input use.
- Do not use the M2 sample as proof of low-latency streaming, throughput, frame
  pacing, zero-copy capture-to-encode, no CPU readback, direct DMA-BUF import
  into NVENC, HDR/color behavior, or Mac presentation/render timing.
- Add a follow-up before productizing runtime encoding:
  `m2-encode-parameter-validation` or equivalent. Make constructors fallible or
  use validated newtypes for nonzero dimensions/frame rate/GOP and bounded CQ.
  Include tests for rejected zero dimensions/rate/GOP and out-of-range CQ.
- Existing follow-up/non-claim nodes are appropriate for direct DMA-BUF import,
  no-readback, streaming performance, HDR/color, and Mac decode/render
  boundaries. No new P0/P1 follow-up is required from this audit.
- Orchestrator action: fix or migrate the qd DB outside this subreport if
  `qd node show` output is required for final audit bookkeeping.

## Commands Run

```text
pwd
git status --short --branch
sed -n '1,220p' docs/ORCHESTRATION.md
sed -n '1,260p' docs/LINUX_ORCHESTRATOR.md
qd node show m2-audit --full
rg -n "m2-audit|m2-nvenc-encode-sample|m2-video-path-validation" roadmap docs evidence reports crates -S
find evidence reports -maxdepth 4 -type f | sort
rg --files crates/encode crates/encode-nv-sys tests 2>/dev/null
rg -n "unsafe|Command|ffmpeg|ffprobe|nvenc|NVENC|AV1|low.lat|hash|sha|sample|readback|zero.copy|ivf|Annex|NAL|size" crates/encode crates/encode-nv-sys tests evidence reports -S
nl -b a crates/encode-nv-sys/src/lib.rs
nl -b a crates/encode/src/lib.rs
jq '.nodes[] | select(.id=="m2-nvenc-encode-sample" or .id=="m2-video-path-validation" or .id=="m2-audit")' roadmap/qd-export.json
jq '.findings[]? | select(.node_id=="m2-video-path-validation" or .node_id=="m2-nvenc-encode-sample" or .node_id=="m2-audit")' roadmap/qd-export.json
nl -b a evidence/m2-nvenc-encode-sample/notes.md
nl -b a evidence/m2-video-path-validation/bundle-manifest.json
nl -b a evidence/m2-video-path-validation/notes.md
nl -b a reports/qd/m2-nvenc-encode-sample/completion.json
nl -b a reports/qd/m2-video-path-validation/completion.json
nl -b a reports/qd/m2-nvenc-encode-sample/audit.json
nl -b a reports/qd/m2-video-path-validation/audit.json
nl -b a evidence/m2-nvenc-encode-sample/validation-summary.json
nl -b a evidence/m2-nvenc-encode-sample/commands.log
nl -b a evidence/m2-nvenc-encode-sample/unsafe-boundary-scan.log
sha256sum evidence/m2-nvenc-encode-sample/sample-av1.ivf
wc -c evidence/m2-nvenc-encode-sample/sample-av1.ivf
xxd -g1 -l 64 evidence/m2-nvenc-encode-sample/sample-av1.ivf
ffprobe -v error -show_streams -show_format -of json evidence/m2-nvenc-encode-sample/sample-av1.ivf
nl -b a evidence/m3-videotoolbox-decode-sample/decode-report.json
nl -b a evidence/m3-videotoolbox-decode-sample/sample-input-hash.txt
nl -b a crates/transport/src/video_smoke/mod.rs | sed -n '1,220p'
nl -b a crates/transport/tests/lan_video_smoke.rs | sed -n '1,180p'
nl -b a reports/qd/m3-videotoolbox-decode-sample/completion.json | sed -n '1,120p'
nl -b a evidence/m3-lan-video-smoke-harness/result.json
nl -b a evidence/m3-lan-video-smoke-harness/sample-hash.txt
nl -b a crates/transport/src/video_smoke/mod.rs | sed -n '213,360p'
rg -n "LinuxNvencAv1Encoder|DesktopAv1EncodeSettings|CapturedSampleFrame|FfmpegNvenc|NvencAv1Request|sample_command|av1_command" -S .
nl -b a evidence/m2-video-path-validation-does-not-prove-direct-dma-buf-import-in/report.md
nl -b a evidence/m2-video-path-validation-does-not-measure-streaming-latency-or-p/report.md
nl -b a evidence/m2-video-path-validation-does-not-prove-no-cpu-readback-after-gr/report.md
git status --short
find reports/qd/m2-audit evidence/m2-audit -maxdepth 3 -type f 2>/dev/null | sort
mkdir -p reports/qd/m2-audit/subreports
git diff -- reports/qd/m2-audit/subreports/nvenc-boundary.md
git diff --check
wc -l reports/qd/m2-audit/subreports/nvenc-boundary.md
sed -n '1,240p' reports/qd/m2-audit/subreports/nvenc-boundary.md
git -C /home/trevor/projects/madobe status --short reports/qd/m2-audit/subreports/nvenc-boundary.md .qd/worktrees/m2-audit/reports/qd/m2-audit/subreports/nvenc-boundary.md
test ! -e /home/trevor/projects/madobe/reports/qd/m2-audit/subreports/nvenc-boundary.md
```
