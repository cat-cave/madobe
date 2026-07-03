import CryptoKit
import Foundation

public enum VideoReceiveQueueError: Error, Equatable, Sendable {
    case payloadSizeMismatch(expected: UInt32, actual: Int)
    case payloadHashMismatch(expected: String, actual: String)
}

public struct ReceivedEncodedVideoFrame: Equatable, Sendable {
    public let metadata: EncodedVideoFrameMetadata
    public let payload: Data
    public let receivedTimestampNs: UInt64

    public init(
        metadata: EncodedVideoFrameMetadata,
        payload: Data,
        receivedTimestampNs: UInt64
    ) {
        self.metadata = metadata
        self.payload = payload
        self.receivedTimestampNs = receivedTimestampNs
    }
}

public struct VideoReceiveQueueResult: Equatable, Sendable {
    public let acceptedFrameId: UInt64
    public let droppedFrameId: UInt64?

    public init(acceptedFrameId: UInt64, droppedFrameId: UInt64?) {
        self.acceptedFrameId = acceptedFrameId
        self.droppedFrameId = droppedFrameId
    }
}

public struct VideoReceiveTimelineEvent: Codable, Equatable, Sendable {
    public enum Kind: String, Codable, Equatable, Sendable {
        case received
        case dropped
        case dequeued
    }

    public let kind: Kind
    public let frameId: UInt64
    public let timestampNs: UInt64

    public init(kind: Kind, frameId: UInt64, timestampNs: UInt64) {
        self.kind = kind
        self.frameId = frameId
        self.timestampNs = timestampNs
    }
}

public struct VideoReceiveTimeline: Codable, Equatable, Sendable {
    public let events: [VideoReceiveTimelineEvent]

    public init(events: [VideoReceiveTimelineEvent]) {
        self.events = events
    }

    public func jsonData() throws -> Data {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return try encoder.encode(self)
    }
}

public final class VideoReceiveQueue {
    private var readyFrame: ReceivedEncodedVideoFrame?
    private var events: [VideoReceiveTimelineEvent] = []

    public init() {}

    public var readyCount: Int {
        readyFrame == nil ? 0 : 1
    }

    public var timeline: VideoReceiveTimeline {
        VideoReceiveTimeline(events: events)
    }

    @discardableResult
    public func accept(
        metadata: EncodedVideoFrameMetadata,
        payload: Data,
        receivedTimestampNs: UInt64
    ) throws -> VideoReceiveQueueResult {
        try Self.validatePayload(metadata: metadata, payload: payload)

        let droppedFrameId = readyFrame?.metadata.frameId
        if let droppedFrameId {
            events.append(
                VideoReceiveTimelineEvent(
                    kind: .dropped,
                    frameId: droppedFrameId,
                    timestampNs: receivedTimestampNs
                )
            )
        }

        let frame = ReceivedEncodedVideoFrame(
            metadata: metadata,
            payload: payload,
            receivedTimestampNs: receivedTimestampNs
        )
        readyFrame = frame
        events.append(
            VideoReceiveTimelineEvent(
                kind: .received,
                frameId: metadata.frameId,
                timestampNs: receivedTimestampNs
            )
        )

        return VideoReceiveQueueResult(
            acceptedFrameId: metadata.frameId,
            droppedFrameId: droppedFrameId
        )
    }

    public func dequeueForDecode(dequeueTimestampNs: UInt64) -> ReceivedEncodedVideoFrame? {
        guard let frame = readyFrame else {
            return nil
        }

        readyFrame = nil
        events.append(
            VideoReceiveTimelineEvent(
                kind: .dequeued,
                frameId: frame.metadata.frameId,
                timestampNs: dequeueTimestampNs
            )
        )
        return frame
    }

    private static func validatePayload(
        metadata: EncodedVideoFrameMetadata,
        payload: Data
    ) throws {
        let actualPayloadBytes = UInt32(exactly: payload.count)
        if metadata.payloadBytes != actualPayloadBytes {
            throw VideoReceiveQueueError.payloadSizeMismatch(
                expected: metadata.payloadBytes,
                actual: payload.count
            )
        }

        let actualHash = sha256Hex(payload)
        if metadata.payloadHash.value != actualHash {
            throw VideoReceiveQueueError.payloadHashMismatch(
                expected: metadata.payloadHash.value,
                actual: actualHash
            )
        }
    }

    private static func sha256Hex(_ payload: Data) -> String {
        SHA256.hash(data: payload)
            .map { String(format: "%02x", $0) }
            .joined()
    }
}
