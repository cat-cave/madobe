import MadobeClientCore
import XCTest

final class MadobeClientCoreTests: XCTestCase {
    private static let expectedEncodedFrame = EncodedVideoFrameMetadata(
        frameId: 42,
        codec: .av1,
        width: 2560,
        height: 1440,
        captureTimestampNs: 1_720_000_000_000_000_000,
        encodeTimestampNs: 1_720_000_000_004_166_667,
        keyframe: true,
        payloadBytes: 38,
        payloadHash: PayloadHash(
            algorithm: .sha256,
            value: "4808d39bb0065087388612224cfda59f52f0278772f390065aa5e743f7bc0667"
        )
    )

    func testIdentityMatchesRustBootstrapProof() {
        XCTAssertEqual(
            MadobeClientCore.identity().statusLine,
            "madobe 0.1.0 protocol=1 event=madobe.bootstrap ts=0 status=ok"
        )
    }

    func testEncodedVideoFrameFixtureMatchesRustGoldenVector() throws {
        let fixtureData = try Self.loadRustEncodedVideoFrameFixture()
        let decodedFrame = try JSONDecoder().decode(
            EncodedVideoFrameMetadata.self,
            from: fixtureData
        )

        XCTAssertEqual(decodedFrame, Self.expectedEncodedFrame)
        XCTAssertEqual(decodedFrame.frameId, 42)
        XCTAssertEqual(decodedFrame.codec, .av1)
        XCTAssertEqual(decodedFrame.width, 2560)
        XCTAssertEqual(decodedFrame.height, 1440)
        XCTAssertEqual(decodedFrame.captureTimestampNs, 1_720_000_000_000_000_000)
        XCTAssertEqual(decodedFrame.encodeTimestampNs, 1_720_000_000_004_166_667)
        XCTAssertTrue(decodedFrame.keyframe)
        XCTAssertEqual(decodedFrame.payloadBytes, 38)
        XCTAssertEqual(decodedFrame.payloadHash.algorithm, .sha256)
        XCTAssertEqual(
            decodedFrame.payloadHash.value,
            "4808d39bb0065087388612224cfda59f52f0278772f390065aa5e743f7bc0667"
        )
    }

    func testEncodedVideoFrameRejectsUnknownCodecFixtureMismatch() throws {
        let fixtureJSON = try Self.loadRustEncodedVideoFrameFixtureString()
        let mismatchedJSON = fixtureJSON.replacingOccurrences(
            of: "\"codec\": \"av1\"",
            with: "\"codec\": \"h264\""
        )
        let mismatchedData = try XCTUnwrap(mismatchedJSON.data(using: .utf8))

        XCTAssertThrowsError(
            try JSONDecoder().decode(EncodedVideoFrameMetadata.self, from: mismatchedData)
        ) { error in
            guard case DecodingError.dataCorrupted = error else {
                return XCTFail("Expected dataCorrupted for unknown codec, got \(error)")
            }
        }
    }

    func testEncodedVideoFramePayloadHashMismatchChangesSemantics() throws {
        let fixtureJSON = try Self.loadRustEncodedVideoFrameFixtureString()
        let mismatchedJSON = fixtureJSON.replacingOccurrences(
            of: "4808d39bb0065087388612224cfda59f52f0278772f390065aa5e743f7bc0667",
            with: "0000000000000000000000000000000000000000000000000000000000000000"
        )
        let mismatchedData = try XCTUnwrap(mismatchedJSON.data(using: .utf8))
        let decodedFrame = try JSONDecoder().decode(
            EncodedVideoFrameMetadata.self,
            from: mismatchedData
        )

        XCTAssertNotEqual(decodedFrame, Self.expectedEncodedFrame)
        XCTAssertNotEqual(
            decodedFrame.payloadHash.value,
            Self.expectedEncodedFrame.payloadHash.value
        )
    }

    private static func loadRustEncodedVideoFrameFixtureString(
        filePath: String = #filePath
    ) throws -> String {
        let data = try loadRustEncodedVideoFrameFixture(filePath: filePath)
        return try XCTUnwrap(String(bytes: data, encoding: .utf8))
    }

    private static func loadRustEncodedVideoFrameFixture(
        filePath: String = #filePath
    ) throws -> Data {
        var root = URL(fileURLWithPath: filePath)
        for _ in 0 ..< 4 {
            root.deleteLastPathComponent()
        }

        let fixtureURL = root.appendingPathComponent(
            "crates/protocol/fixtures/encoded-video-frame-av1.json"
        )
        return try Data(contentsOf: fixtureURL)
    }
}
