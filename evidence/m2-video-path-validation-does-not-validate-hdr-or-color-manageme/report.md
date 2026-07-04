# m2-video-path-validation HDR/color non-claim audit

Node: `m2-video-path-validation-does-not-validate-hdr-or-color-manageme`

## Conclusion

Status: passed.

The M2 validation bundle now has an explicit, machine-checkable HDR/color boundary. The existing encoded sample records
`pixelFormat=yuv420p`, `colorRange=tv`, and unknown `colorSpace`, `colorTransfer`, and `colorPrimaries`.

The sample is decoder-consumable AV1/IVF evidence only. It does not validate HDR handling, color-management behavior,
wide-gamut preservation, or end-to-end color accuracy.

## Files and strings reviewed

- `evidence/m2-video-path-validation/bundle-manifest.json`: adds `evidenceBoundary.sampleColorMetadata` with the
  observed ffprobe color fields.
- `evidence/m2-video-path-validation/bundle-manifest.json`: adds machine-checkable false claims for HDR handling,
  color-management behavior, wide-gamut preservation, and end-to-end color accuracy.
- `evidence/m2-video-path-validation/bundle-manifest.json`: lists HDR/color behavior, wide-gamut preservation, and
  end-to-end color accuracy under `notEvidenceFor` and `explicitNonClaims`.
- `evidence/m2-video-path-validation/notes.md`: spells out the same HDR/color non-claim in prose.
- `evidence/m2-nvenc-encode-sample/notes.md`: prevents the source encode sample from being treated as HDR/color proof.

## Future work boundary

Future HDR/color validation would need appropriate source material, tagged color metadata, and checks for HDR handling,
color-management behavior, wide-gamut preservation, and end-to-end color accuracy. That validation is future work and is
not performed in this node.

## Commands run

See `commands.log` and `json-validation.log` in this node evidence directory.

The orchestrator also ran `qd gate m2-video-path-validation-does-not-validate-hdr-or-color-manageme --phase ci` and
`nix develop -c just verify`; both passed and their logs are recorded in this evidence directory.
