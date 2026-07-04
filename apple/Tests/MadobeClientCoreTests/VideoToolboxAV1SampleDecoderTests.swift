import Foundation
import MadobeClientCore
import VideoToolbox
import XCTest

final class VideoToolboxAV1SampleDecoderTests: XCTestCase {
    func testParsesLinuxAV1IVFSample() throws {
        let sampleURL = Self.sampleURL()
        let parsed = try VideoToolboxAV1SampleDecoder.parseIVFSample(
            Data(contentsOf: sampleURL)
        )

        XCTAssertEqual(parsed.width, 160)
        XCTAssertEqual(parsed.height, 90)
        XCTAssertEqual(parsed.timescale, 60)
        XCTAssertEqual(parsed.frameCount, 1)
        XCTAssertEqual(parsed.framePayloadBytes, 40)
        XCTAssertEqual(parsed.samplePayloadBytes, 38)
        XCTAssertEqual(
            parsed.av1C.map { String(format: "%02x", $0) }.joined(),
            "81000c000a0a00000003b4fd90086601"
        )
    }

    func testDecodesLinuxAV1IVFSampleWhenHardwareAvailable() throws {
        guard VTIsHardwareDecodeSupported(kCMVideoCodecType_AV1) else {
            throw XCTSkip("VideoToolbox AV1 hardware decode is unsupported on this runner.")
        }

        let report = try VideoToolboxAV1SampleDecoder.decodeIVFSample(
            at: Self.sampleURL(),
            samplePath: "evidence/m2-nvenc-encode-sample/sample-av1.ivf"
        )
        try Self.writeDecodeReportIfRequested(report)

        XCTAssertEqual(
            report.sampleSHA256,
            "51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875"
        )
        XCTAssertEqual(report.hardwareDecodeSupported, true)

        switch report.decodeStatus {
        case "decoded":
            XCTAssertNil(report.unsupportedReason)
            XCTAssertEqual(report.usingHardwareAcceleratedDecoder, true)
            XCTAssertEqual(report.decodedWidth, 160)
            XCTAssertEqual(report.decodedHeight, 90)
            XCTAssertEqual(report.decodedPixelFormat, "420v")
            XCTAssertNotNil(report.decodeDurationNanoseconds)
        case "unsupported":
            let reason = try XCTUnwrap(report.unsupportedReason)
            XCTAssertTrue(reason.contains("VTDecompressionSessionCreate"))
            XCTAssertNil(report.usingHardwareAcceleratedDecoder)
            XCTAssertNil(report.decodedWidth)
            XCTAssertNil(report.decodedHeight)
            XCTAssertNil(report.decodedPixelFormat)
            XCTAssertNil(report.decodeDurationNanoseconds)
        default:
            XCTFail("Unexpected decode status: \(report.decodeStatus)")
        }
    }

    func testDecodeReportJSONIsStableAndDecodable() throws {
        let report = VideoToolboxAV1SampleDecodeReport(
            samplePath: "sample.ivf",
            sampleSHA256: "abc123",
            container: "ivf",
            codec: .av1,
            width: 160,
            height: 90,
            timescale: 60,
            frameCount: 1,
            framePayloadBytes: 40,
            samplePayloadBytes: 38,
            av1C: "81000c000a0a00000003b4fd90086601",
            hardwareDecodeSupported: true,
            decodeStatus: "decoded",
            unsupportedReason: nil,
            usingHardwareAcceleratedDecoder: true,
            decodedWidth: 160,
            decodedHeight: 90,
            decodedPixelFormat: "420v",
            decodeDurationNanoseconds: 1234
        )

        let data = try report.jsonData()
        let decoded = try JSONDecoder().decode(
            VideoToolboxAV1SampleDecodeReport.self,
            from: data
        )
        let json = try XCTUnwrap(String(data: data, encoding: .utf8))

        XCTAssertEqual(decoded, report)
        XCTAssertTrue(json.contains("\"decodeStatus\" : \"decoded\""))
        XCTAssertTrue(json.contains("\"sampleSHA256\" : \"abc123\""))
    }

    private static func sampleURL() -> URL {
        repoRoot()
            .appendingPathComponent("evidence")
            .appendingPathComponent("m2-nvenc-encode-sample")
            .appendingPathComponent("sample-av1.ivf")
    }

    private static func writeDecodeReportIfRequested(
        _ report: VideoToolboxAV1SampleDecodeReport
    ) throws {
        guard let reportPath = requestedDecodeReportPath() else {
            return
        }

        let reportURL = URL(fileURLWithPath: reportPath)
        try FileManager.default.createDirectory(
            at: reportURL.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try report.jsonData().write(to: reportURL, options: .atomic)
    }

    private static func requestedDecodeReportPath() -> String? {
        let environment = ProcessInfo.processInfo.environment
        if let reportPath = environment["MADOBE_AV1_DECODE_REPORT_PATH"] {
            return reportPath
        }

        let markerURL = repoRoot().appendingPathComponent(".madobe-av1-decode-report-path")
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
