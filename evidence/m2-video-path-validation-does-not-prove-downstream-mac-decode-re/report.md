# m2-video-path-validation downstream Mac decode/render non-claim audit

Node: `m2-video-path-validation-does-not-prove-downstream-mac-decode-re`

## Conclusion

Status: passed.

The M2 video path bundle now has an explicit, machine-checkable downstream Mac decode/render boundary. The existing
Linux-produced `sample-av1.ivf` remains decoder-consumable AV1/IVF sample evidence only.

It does not validate VideoToolbox decode, Metal render, Mac presentation/display, Mac frame timing, or cross-device
render behavior.

## Files and strings reviewed

- `evidence/m2-video-path-validation/bundle-manifest.json`: adds false machine-checkable claims for downstream Mac
  decode/render validation, VideoToolbox decode, Metal render, Mac presentation/display, Mac frame timing, and
  cross-device render behavior.
- `evidence/m2-video-path-validation/bundle-manifest.json`: adds
  `evidenceBoundary.downstreamMacDecodeRenderBoundary` with the current positive claim, non-validated areas, and future
  Mac validation requirement.
- `evidence/m2-video-path-validation/notes.md`: spells out that the M2 bundle is not Mac decode/render proof.
- `evidence/m2-nvenc-encode-sample/notes.md`: prevents the source encode sample from being reused as Mac
  decode/render evidence.

## Future work boundary

Positive proof remains future Mac validation work, likely via `m3-videotoolbox-decode-sample`. That future node must
execute VideoToolbox decode and record downstream render/presentation evidence before Mac decode/render claims can be
made.

## Commands run

See `commands.log` and `json-validation.log` in this node evidence directory.

The orchestrator also ran `qd gate m2-video-path-validation-does-not-prove-downstream-mac-decode-re --phase ci` and
`nix develop -c just verify`; both passed and their logs are recorded in this evidence directory.
