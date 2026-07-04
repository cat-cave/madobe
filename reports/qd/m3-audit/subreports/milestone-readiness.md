# M3 Milestone Readiness Audit

## Scope

Audited M3 readiness for `M3-mac-video-endpoint`, focusing on whether the remaining non-claims require new qd nodes before milestone completion or can be carried as future-milestone work.

This subreport reviewed the M3 roadmap state, cross-device runbook, orchestration guidance, M3 evidence/reports, and the M2 downstream-readiness audit. It did not run qd state mutation commands and did not edit files outside this subreport.

## Commands And Files Reviewed

Focused read/check commands:

- `git status --short --branch`
- `rg --files roadmap docs evidence reports | sort`
- `rg -n "M3|m3|QUIC|decode|Metal|render|presentation|latency" roadmap/qd-export.json docs/CROSS_DEVICE_VALIDATION.md docs/ORCHESTRATION.md reports evidence`
- `jq '.nodes[] | select(.milestone=="M3-mac-video-endpoint") | {id,title,status,status_reason,spec,acceptance,branch,context}' roadmap/qd-export.json`
- `sed -n '1,240p' docs/CROSS_DEVICE_VALIDATION.md`
- `sed -n '1,240p' docs/ORCHESTRATION.md`
- `sed -n '1,220p' reports/qd/m2-audit/subreports/downstream-readiness.md`
- `jq -e .` over key M3 result/report JSON artifacts
- `rg -n "not QUIC|productQuic|not VideoToolbox decode|not Metal render|not presentation|not latency|frames_decoded|frames_rendered|frames_presented|median_glass|p95_glass|presentation_evidence|latency_evidence|decode_evidence|render_evidence" docs evidence reports`

Files reviewed:

- `roadmap/qd-export.json`
- `docs/CROSS_DEVICE_VALIDATION.md`
- `docs/ORCHESTRATION.md`
- `docs/LAN_VIDEO_SMOKE_HARNESS.md`
- `docs/LINUX_VIDEO_SMOKE_HOST_PREFLIGHT.md`
- `reports/qd/m2-audit/subreports/downstream-readiness.md`
- `reports/qd/m3-*/completion.json`
- `reports/qd/m3-*/audit.json`
- `evidence/m3-*/notes.md`
- `evidence/m3-cross-device-video-smoke/result.json`
- `evidence/m3-cross-device-video-smoke/macos/linux-attempt/result.json`
- `evidence/m3-cross-device-video-smoke/linux/*`
- `evidence/m3-lan-video-smoke-harness/result.json`
- `evidence/m3-videotoolbox-decode-sample/decode-report.json`
- `evidence/m3-metal-renderer-skeleton/render-report.json`
- `evidence/m3-videotoolbox-capability-probe/capability-report.json`
- `evidence/m3-mac-video-receive-queue/client-receive-timeline.json`
- `evidence/m3-linux-video-smoke-host-preflight/host-preflight-summary.json`

## P0/P1 Blockers

None for the current M3 acceptance boundary.

All M3 nodes except `m3-audit` are `done` in `roadmap/qd-export.json`. The live cross-device smoke passed as a Linux-to-Mac TCP LAN sample-transfer proof: one checked-in 84-byte AV1 IVF sample was sent by Linux and validated by the Mac receiver by metadata, byte count, and SHA-256.

The important readiness point is that the evidence does not overclaim. `evidence/m3-cross-device-video-smoke/result.json` records `frames_sent=1`, `frames_decoded=0`, `frames_rendered=0`, `frames_presented=0`, and null latency. The result notes explicitly exclude product QUIC, VideoToolbox decode, Metal render, presentation, and latency.

## P2 Follow-Up Node Pitches

- Product QUIC cross-device validation: future milestone, not an M3 blocker. M3 has an in-memory QUIC-shaped loopback skeleton and a TCP LAN smoke harness. Add a later product-transport node before claiming real QUIC handshakes, socket I/O, congestion behavior, TLS, stream scheduling, or cross-device product transport.
- Decode received network sample on Mac: recommended qd node before describing the Mac endpoint as network-decode capable. M3 proves local VideoToolbox decode of the checked-in sample and separately proves TCP receipt/hash validation, but not the integrated path from received bytes to VideoToolbox decode.
- Render/present received decoded sample: recommended qd node after network decode. M3 proves a local Metal renderer skeleton/test pattern and local render timing, but not Metal render or visible presentation from the sample received in the live LAN smoke.
- Cross-device frame latency: future milestone. Existing timelines are sequence/event evidence and explicitly do not support glass-to-glass or frame latency metrics. Add a latency-specific node with timeline artifacts and non-null metric evidence before any latency claim.

## P3 Polish

- Rename or clarify `render-report.json` field `presentedTestPattern`; the M2 downstream audit already noted it can be misread as display presentation even though the practical evidence is local/offscreen render timing plus app-shell wiring.
- Tighten cross-device result validation later to verify referenced artifact existence/content, not only metric-to-artifact-kind consistency.
- Normalize sub-result naming under `evidence/m3-cross-device-video-smoke/macos/linux-attempt/result.json`; it currently carries `nodeId: m3-lan-video-smoke-harness`, which is accurate for the harness but mildly confusing inside the cross-device smoke evidence tree.
- Add one short milestone summary doc/comment that states the exact M3 claim boundary: protocol compatibility, local Mac AV1 decode, local Metal skeleton, receive-queue validation, and Linux-to-Mac TCP sample transfer only.

## Milestone Completion Recommendation

Recommend completing M3 only with the conservative claim boundary above. No P0/P1 blocker requires more implementation before M3 completion if the milestone is framed as endpoint foundation plus first cross-device sample transfer.

Do not describe M3 as product video streaming, product QUIC, network-driven VideoToolbox decode, Metal presentation of a received sample, or latency-ready. Those require follow-up nodes. The strongest option is to create/track the P2 follow-up nodes now, but place product QUIC and latency in the next milestone unless leadership intentionally expands M3 scope.

## GitHub Coordination Recommendations

- Post a coordination comment summarizing the M3 claim boundary and link this subreport, `evidence/m3-cross-device-video-smoke/result.json`, and the M2 downstream-readiness subreport.
- Open or track follow-up GitHub issues/qd nodes for product QUIC validation, Mac decode from received sample, Metal render/presentation from received sample, and latency instrumentation.
- Label the follow-ups with `needs:cross-device-validation`, `needs:mac-validation`, and `evidence-needed` as appropriate.
- In any M3 completion PR, include residual-risk language that the live smoke is TCP sample transfer only and that decode/render/presentation/latency metrics intentionally remain zero/null.
