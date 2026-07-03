import CoreMedia
import MadobeClientCore
import VideoToolbox
import XCTest

final class VideoToolboxCapabilityProbeTests: XCTestCase {
    func testReportIncludesRuntimeAV1Capability() throws {
        let report = VideoToolboxCapabilityProbe.report(
            operatingSystemVersion: "test-runtime"
        )

        let av1 = try XCTUnwrap(report.codecs.first { $0.codec == .av1 })
        XCTAssertEqual(report.operatingSystemVersion, "test-runtime")
        XCTAssertEqual(av1.coreMediaFourCharacterCode, "av01")
        XCTAssertEqual(
            av1.hardwareDecodeSupported,
            VTIsHardwareDecodeSupported(kCMVideoCodecType_AV1)
        )
    }

    func testJSONReportIsStableAndDecodable() throws {
        let report = VideoToolboxCapabilityReport(
            operatingSystemVersion: "test-runtime",
            codecs: [
                VideoToolboxCodecCapability(
                    codec: .av1,
                    coreMediaFourCharacterCode: "av01",
                    hardwareDecodeSupported: true
                ),
            ]
        )

        let data = try VideoToolboxCapabilityProbe.jsonData(for: report)
        let decodedReport = try JSONDecoder().decode(
            VideoToolboxCapabilityReport.self,
            from: data
        )

        XCTAssertEqual(decodedReport, report)
        let json = try XCTUnwrap(String(data: data, encoding: .utf8))
        XCTAssertTrue(json.contains("\"av1\""))
    }
}
