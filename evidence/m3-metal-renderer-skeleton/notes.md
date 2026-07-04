# m3-metal-renderer-skeleton Notes

This node adds a minimal Metal renderer path for the macOS app shell and a core render timing probe that clears a BGRA8 render target to a validation-red test pattern.

Evidence summary:

- App shell presentation: `MetalTestPatternView` is an `MTKView` wrapper wired into `ContentView`.
- Render probe: `MetalRenderTimingProbe.renderTestPatternFrame` creates a Metal device, command queue, BGRA8 render target, render pass, command buffer, and CPU-readable sample of the first rendered pixel.
- Render report: historical raw `render-report.json` records device `Apple M5`, `commandBufferStatus` `completed`, `presentedTestPattern` true, sampled BGRA8 pixel `[0, 0, 255, 255]`, and `renderDurationNanoseconds`. The legacy `presentedTestPattern` field means the offscreen render target contained the test pattern; it is not display/drawable presentation evidence.
- macOS validation: `xcodebuild.log` records the focused Xcode test run that generated `render-report.json`.
- Full gate: `macos-check.log` records the passing `just macos-check` run.

Important limits:

- This node does not claim live streaming, cross-device validation, end-to-end latency, decoded-frame scheduling, HDR/color-management behavior, or Linux host rendering/capture behavior.
- A GUI screenshot was not captured because this was a noninteractive orchestration run and a full-screen desktop capture would risk committing unrelated local screen content. The practical visual evidence is the app shell `MTKView` path plus the offscreen Metal hardware render report; display presentation requires a later drawable, screenshot, or equivalent presentation artifact.

Non-blocking warnings:

- `xcodebuild` reports multiple matching macOS destinations and selects the arm64 `My Mac` destination.
- `appintentsmetadataprocessor` reports metadata extraction skipped because this app has no AppIntents.framework dependency.
