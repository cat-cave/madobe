import Foundation
import MadobeClientCore
import XCTest

final class MetalRenderTimingProbeTests: XCTestCase {
    func testRendersValidationTestPatternWhenMetalIsAvailable() throws {
        guard MetalRenderTimingProbe.isSupported else {
            throw XCTSkip("Metal is unavailable on this runner.")
        }

        let report = try MetalRenderTimingProbe.renderTestPatternFrame(
            width: 64,
            height: 36,
            clearColor: .validationRed
        )
        try Self.writeRenderReportIfRequested(report)

        XCTAssertEqual(report.renderer, "metal-clear-test-pattern")
        XCTAssertEqual(report.width, 64)
        XCTAssertEqual(report.height, 36)
        XCTAssertEqual(report.pixelFormat, "bgra8Unorm")
        XCTAssertEqual(report.renderTargetKind, "offscreen-texture")
        XCTAssertEqual(report.clearColor, .validationRed)
        XCTAssertEqual(report.sampledPixelBGRA8, [0, 0, 255, 255])
        XCTAssertEqual(report.commandBufferStatus, "completed")
        XCTAssertEqual(report.offscreenTestPatternRendered, true)
        XCTAssertEqual(report.displayPresented, false)
        XCTAssertGreaterThan(report.renderDurationNanoseconds, 0)
    }

    func testRenderTimingReportJSONIsStableAndDecodable() throws {
        let report = MetalRenderTimingReport(
            renderer: "metal-clear-test-pattern",
            deviceName: "Test GPU",
            width: 64,
            height: 36,
            pixelFormat: "bgra8Unorm",
            renderTargetKind: "offscreen-texture",
            clearColor: .validationRed,
            sampledPixelBGRA8: [0, 0, 255, 255],
            commandBufferStatus: "completed",
            offscreenTestPatternRendered: true,
            displayPresented: false,
            renderDurationNanoseconds: 1234
        )

        let data = try report.jsonData()
        let decoded = try JSONDecoder().decode(MetalRenderTimingReport.self, from: data)
        let json = try XCTUnwrap(String(data: data, encoding: .utf8))

        XCTAssertEqual(decoded, report)
        XCTAssertTrue(json.contains("\"renderer\" : \"metal-clear-test-pattern\""))
        XCTAssertTrue(json.contains("\"pixelFormat\" : \"bgra8Unorm\""))
        XCTAssertTrue(json.contains("\"renderTargetKind\" : \"offscreen-texture\""))
        XCTAssertTrue(json.contains("\"offscreenTestPatternRendered\" : true"))
        XCTAssertTrue(json.contains("\"displayPresented\" : false"))
        XCTAssertFalse(json.contains("presentedTestPattern"))
    }

    private static func writeRenderReportIfRequested(
        _ report: MetalRenderTimingReport
    ) throws {
        guard let reportPath = requestedRenderReportPath() else {
            return
        }

        let reportURL = URL(fileURLWithPath: reportPath)
        try FileManager.default.createDirectory(
            at: reportURL.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try report.jsonData().write(to: reportURL, options: .atomic)
    }

    private static func requestedRenderReportPath() -> String? {
        let environment = ProcessInfo.processInfo.environment
        if let reportPath = environment["MADOBE_METAL_RENDER_REPORT_PATH"] {
            return reportPath
        }

        let markerURL = repoRoot().appendingPathComponent(".madobe-metal-render-report-path")
        guard let marker = try? String(contentsOf: markerURL) else {
            return nil
        }
        let reportPath = marker.trimmingCharacters(in: .whitespacesAndNewlines)
        return reportPath.isEmpty ? nil : reportPath
    }

    private static func repoRoot() -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
    }
}
