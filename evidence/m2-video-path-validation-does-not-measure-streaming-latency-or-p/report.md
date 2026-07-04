# m2-video-path-validation streaming performance non-claim audit

Node: `m2-video-path-validation-does-not-measure-streaming-latency-or-p`

Date: July 3, 2026

## Conclusion

Status: passed.

The M2 validation bundle now has an explicit, machine-checkable streaming performance boundary. The existing evidence is
a one-frame AV1/IVF sample and command logs only. It is decoder-consumable sample evidence, not performance evidence.

The bundle does not measure or validate end-to-end streaming latency, encode latency, throughput, frame pacing/jitter,
CPU utilization, GPU utilization, network behavior, or cross-device latency.

## Files and strings reviewed

- `evidence/m2-video-path-validation/bundle-manifest.json`: adds machine-checkable false claims for streaming latency,
  encode latency, throughput, frame pacing/jitter, CPU utilization, GPU utilization, network behavior, and cross-device
  latency.
- `evidence/m2-video-path-validation/bundle-manifest.json`: lists those performance dimensions under
  `evidenceBoundary.notEvidenceFor` and `explicitNonClaims`.
- `evidence/m2-video-path-validation/notes.md`: records that the sample, command logs, and `av1_nvenc` low-latency tune
  setting must not be treated as performance evidence.
- `evidence/m2-nvenc-encode-sample/notes.md`: records that the one-frame encode sample does not validate streaming
  performance and that `tune=ll` is only a configured encoder mode.

## Future work boundary

Future streaming performance validation must run an end-to-end stream with instrumentation for streaming latency, encode
latency, throughput, frame pacing/jitter, CPU utilization, GPU utilization, network behavior, and cross-device latency as
applicable. That validation is future work and is not performed in this node.

## Commands run

See `commands.log` and `json-validation.log` in this node evidence directory.

The orchestrator also ran `qd gate m2-video-path-validation-does-not-measure-streaming-latency-or-p --phase ci` and
`nix develop -c just verify`; both passed and their logs are recorded in this evidence directory.
