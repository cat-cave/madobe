# m2-video-path-validation no-CPU-readback non-claim audit

Node: `m2-video-path-validation-does-not-prove-no-cpu-readback-after-gr`

Date: July 3, 2026

## Conclusion

Status: passed.

The repo does not treat the grim-based encode sample as no-CPU-readback or zero-copy evidence. The validation bundle
already recorded the non-claim. The upstream encode-sample note described the grim PNG and raw RGB materialization, but
needed a direct non-claim sentence, so it was patched narrowly.

## Files and strings reviewed

- `evidence/m2-nvenc-encode-sample/notes.md:9-11`: records that a node-scoped Hyprland output was captured with
  `grim`, then the captured PNG was converted to one deterministic `rgb24` raw frame and encoded through ffmpeg/NVENC.
- `evidence/m2-nvenc-encode-sample/notes.md:17-19`: now records that `captured-sample.png` and
  `captured-sample.rgb` are materialized diagnostic sample inputs and do not prove no CPU readback, zero-copy capture,
  or direct capture-to-NVENC DMA-BUF transfer.
- `evidence/m2-nvenc-encode-sample/validation-summary.json`: records `sample.source` as
  `grim capture from node-scoped Hyprland headless output`, `sample.inputPng` as `captured-sample.png`, and
  `sample.rawFrame` as `captured-sample.rgb`.
- `evidence/m2-video-path-validation/notes.md:28-39`: the Explicit Non-Claims section includes
  `no CPU readback after the grim-based encode sample` and states that the upstream sample uses a grim PNG/RGB capture
  path plus an ffmpeg/NVENC process boundary.
- `evidence/m2-video-path-validation/bundle-manifest.json:96-100`: records the encode summary source as grim capture,
  with `captured-sample.png`, `captured-sample.rgb`, and `rgb24`.
- `evidence/m2-video-path-validation/bundle-manifest.json:209-215`: `explicitNonClaims` includes
  `This bundle does not prove no CPU readback after the grim-based encode sample.`
- `evidence/m2-video-path-validation/bundle-manifest.json:235-236`: the known qd finding states that the encode sample
  explicitly materializes `captured-sample.png` and `captured-sample.rgb`, and that no zero-copy or no-CPU-readback
  claim should be made for the grim-based sample.

## Wording patched

Patched `evidence/m2-nvenc-encode-sample/notes.md` only. No code, topology, roadmap state, qd completion state, push, or
PR action was changed.

Added wording:

```text
The grim PNG and raw RGB artifacts (`captured-sample.png` and `captured-sample.rgb`) are materialized diagnostic
sample inputs for this encode proof. They do not prove no CPU readback, zero-copy capture, or direct capture-to-NVENC
DMA-BUF transfer.
```

## Commands run

```text
rg -n "materialized diagnostic|does not prove no CPU readback|zero-copy|direct capture-to-NVENC|grim PNG/RGB|captured-sample\.png|captured-sample\.rgb|no CPU readback after the grim-based encode sample" evidence/m2-nvenc-encode-sample evidence/m2-video-path-validation reports/qd/m2-video-path-validation
```

Result: passed. The search found the patched encode-sample non-claim plus the existing validation bundle non-claims and
known finding language.

```text
jq -e '.explicitNonClaims[] | select(test("no CPU readback after the grim-based encode sample"))' evidence/m2-video-path-validation/bundle-manifest.json
```

Result: passed. The manifest contains the explicit no-CPU-readback-after-grim non-claim.

```text
jq -e '.sample.inputPng == "captured-sample.png" and .sample.rawFrame == "captured-sample.rgb" and .sample.source == "grim capture from node-scoped Hyprland headless output"' evidence/m2-nvenc-encode-sample/validation-summary.json
```

Result: passed. The upstream encode summary records the materialized grim PNG and raw RGB sample path.

```text
nix develop -c just verify
```

Result: passed. The repo verification completed successfully, including formatting checks, cargo check, clippy,
markdown lint, line lint, 38 nextest tests, doc tests, cargo-deny, cargo-vet, cargo-audit, cargo-machete, and
cargo-semver-checks. Apple tests were skipped outside macOS.
