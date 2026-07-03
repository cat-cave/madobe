import Foundation

public enum VideoCodec: String, Codable, Equatable, Sendable {
    case av1
}

public enum PayloadHashAlgorithm: String, Codable, Equatable, Sendable {
    case sha256
}

public struct PayloadHash: Codable, Equatable, Sendable {
    public let algorithm: PayloadHashAlgorithm
    public let value: String

    public init(algorithm: PayloadHashAlgorithm, value: String) {
        self.algorithm = algorithm
        self.value = value
    }
}

public struct EncodedVideoFrameMetadata: Codable, Equatable, Sendable {
    public let frameId: UInt64
    public let codec: VideoCodec
    public let width: UInt32
    public let height: UInt32
    public let captureTimestampNs: UInt64
    public let encodeTimestampNs: UInt64
    public let keyframe: Bool
    public let payloadBytes: UInt32
    public let payloadHash: PayloadHash

    public init(
        frameId: UInt64,
        codec: VideoCodec,
        width: UInt32,
        height: UInt32,
        captureTimestampNs: UInt64,
        encodeTimestampNs: UInt64,
        keyframe: Bool,
        payloadBytes: UInt32,
        payloadHash: PayloadHash
    ) {
        self.frameId = frameId
        self.codec = codec
        self.width = width
        self.height = height
        self.captureTimestampNs = captureTimestampNs
        self.encodeTimestampNs = encodeTimestampNs
        self.keyframe = keyframe
        self.payloadBytes = payloadBytes
        self.payloadHash = payloadHash
    }
}
