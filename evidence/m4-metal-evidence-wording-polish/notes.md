# m4-metal-evidence-wording-polish Notes

This node clarifies Metal renderer evidence wording so local offscreen render-target proof cannot be mistaken for onscreen presentation evidence.

Changes:

- `MetalRenderTimingReport` now records `renderTargetKind`, `offscreenTestPatternRendered`, and `displayPresented`.
- `MetalRenderTimingProbe.renderTestPatternFrame` reports `renderTargetKind: "offscreen-texture"`, `offscreenTestPatternRendered: true`, and `displayPresented: false`.
- `MetalRenderTimingProbeTests` asserts the clarified fields and verifies that new JSON no longer contains `presentedTestPattern`.
- Existing M3 renderer notes/completion/audit text now annotates historical raw `presentedTestPattern` evidence as offscreen test-pattern validation, not display/drawable presentation.

Evidence:

- `render-report.json` records an Apple M5 offscreen BGRA8 render target, completed command buffer, sampled red BGRA8 pixel `[0, 0, 255, 255]`, `offscreenTestPatternRendered: true`, and `displayPresented: false`.
- `xcodebuild-metal-probe.log` records the focused passing Metal test run.
- `macos-check.log` records the full passing macOS gate with 16 Xcode tests.

Validation boundary:

- This node proves clarified local Metal offscreen render-target evidence wording.
- It does not claim live network rendering, product QUIC, received-frame presentation, screenshot evidence, or cross-device latency.
