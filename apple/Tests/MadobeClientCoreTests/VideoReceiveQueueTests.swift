import CryptoKit
import MadobeClientCore
import XCTest

final class VideoReceiveQueueTests: XCTestCase {
    func testAcceptsAndDequeuesValidFrame() throws {
        let queue = VideoReceiveQueue()
        let payload = Data("frame-one".utf8)
        let metadata = Self.metadata(frameId: 1, payload: payload)

        let result = try queue.accept(
            metadata: metadata,
            payload: payload,
            receivedTimestampNs: 1000
        )
        let received = try XCTUnwrap(
            queue.dequeueForDecode(dequeueTimestampNs: 1200)
        )

        XCTAssertEqual(result, VideoReceiveQueueResult(acceptedFrameId: 1, droppedFrameId: nil))
        XCTAssertEqual(received.metadata, metadata)
        XCTAssertEqual(received.payload, payload)
        XCTAssertEqual(received.receivedTimestampNs, 1000)
        XCTAssertEqual(queue.readyCount, 0)
        XCTAssertEqual(
            queue.timeline.events,
            [
                VideoReceiveTimelineEvent(kind: .received, frameId: 1, timestampNs: 1000),
                VideoReceiveTimelineEvent(kind: .dequeued, frameId: 1, timestampNs: 1200),
            ]
        )
    }

    func testRejectsPayloadByteCountMismatch() throws {
        let queue = VideoReceiveQueue()
        let payload = Data("short".utf8)
        var metadata = Self.metadata(frameId: 2, payload: payload)
        metadata = EncodedVideoFrameMetadata(
            frameId: metadata.frameId,
            codec: metadata.codec,
            width: metadata.width,
            height: metadata.height,
            captureTimestampNs: metadata.captureTimestampNs,
            encodeTimestampNs: metadata.encodeTimestampNs,
            keyframe: metadata.keyframe,
            payloadBytes: 99,
            payloadHash: metadata.payloadHash
        )

        XCTAssertThrowsError(
            try queue.accept(
                metadata: metadata,
                payload: payload,
                receivedTimestampNs: 2000
            )
        ) { error in
            XCTAssertEqual(
                error as? VideoReceiveQueueError,
                .payloadSizeMismatch(expected: 99, actual: 5)
            )
        }
        XCTAssertEqual(queue.readyCount, 0)
        XCTAssertTrue(queue.timeline.events.isEmpty)
    }

    func testRejectsPayloadHashMismatch() throws {
        let queue = VideoReceiveQueue()
        let payload = Data("frame-three".utf8)
        let mismatchedHash = PayloadHash(
            algorithm: .sha256,
            value: String(repeating: "0", count: 64)
        )
        let metadata = Self.metadata(
            frameId: 3,
            payload: payload,
            payloadHash: mismatchedHash
        )

        XCTAssertThrowsError(
            try queue.accept(
                metadata: metadata,
                payload: payload,
                receivedTimestampNs: 3000
            )
        ) { error in
            guard case let .payloadHashMismatch(expected,
                                                actual) = error as? VideoReceiveQueueError
            else {
                return XCTFail("Expected payloadHashMismatch, got \(error)")
            }
            XCTAssertEqual(expected, String(repeating: "0", count: 64))
            XCTAssertEqual(actual.count, 64)
            XCTAssertNotEqual(actual, expected)
        }
        XCTAssertEqual(queue.readyCount, 0)
        XCTAssertTrue(queue.timeline.events.isEmpty)
    }

    func testQueueDepthOneDropsStaleReadyFrame() throws {
        let queue = VideoReceiveQueue()
        let firstPayload = Data("first".utf8)
        let secondPayload = Data("second".utf8)

        try queue.accept(
            metadata: Self.metadata(frameId: 4, payload: firstPayload),
            payload: firstPayload,
            receivedTimestampNs: 4000
        )
        let result = try queue.accept(
            metadata: Self.metadata(frameId: 5, payload: secondPayload),
            payload: secondPayload,
            receivedTimestampNs: 5000
        )
        let received = try XCTUnwrap(
            queue.dequeueForDecode(dequeueTimestampNs: 5100)
        )

        XCTAssertEqual(result, VideoReceiveQueueResult(acceptedFrameId: 5, droppedFrameId: 4))
        XCTAssertEqual(received.metadata.frameId, 5)
        XCTAssertEqual(queue.readyCount, 0)
        XCTAssertEqual(
            queue.timeline.events,
            [
                VideoReceiveTimelineEvent(kind: .received, frameId: 4, timestampNs: 4000),
                VideoReceiveTimelineEvent(kind: .dropped, frameId: 4, timestampNs: 5000),
                VideoReceiveTimelineEvent(kind: .received, frameId: 5, timestampNs: 5000),
                VideoReceiveTimelineEvent(kind: .dequeued, frameId: 5, timestampNs: 5100),
            ]
        )
    }

    func testTimelineJSONIsStableAndDecodable() throws {
        let timeline = VideoReceiveTimeline(
            events: [
                VideoReceiveTimelineEvent(kind: .received, frameId: 8, timestampNs: 8000),
                VideoReceiveTimelineEvent(kind: .dequeued, frameId: 8, timestampNs: 8100),
            ]
        )

        let data = try timeline.jsonData()
        let json = try XCTUnwrap(String(data: data, encoding: .utf8))
        let decoded = try JSONDecoder().decode(VideoReceiveTimeline.self, from: data)

        XCTAssertEqual(decoded, timeline)
        XCTAssertEqual(
            json,
            """
            {
              "events" : [
                {
                  "frameId" : 8,
                  "kind" : "received",
                  "timestampNs" : 8000
                },
                {
                  "frameId" : 8,
                  "kind" : "dequeued",
                  "timestampNs" : 8100
                }
              ]
            }
            """
        )
    }

    private static func metadata(
        frameId: UInt64,
        payload: Data,
        payloadHash: PayloadHash? = nil
    ) -> EncodedVideoFrameMetadata {
        EncodedVideoFrameMetadata(
            frameId: frameId,
            codec: .av1,
            width: 1280,
            height: 720,
            captureTimestampNs: 1_720_000_000_000_000_000 + frameId,
            encodeTimestampNs: 1_720_000_000_002_000_000 + frameId,
            keyframe: frameId == 1,
            payloadBytes: UInt32(payload.count),
            payloadHash: payloadHash ?? PayloadHash(
                algorithm: .sha256,
                value: sha256Hex(payload)
            )
        )
    }

    private static func sha256Hex(_ payload: Data) -> String {
        SHA256.hash(data: payload)
            .map { String(format: "%02x", $0) }
            .joined()
    }
}
