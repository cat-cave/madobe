import CoreMedia
import CoreVideo
import CryptoKit
import Foundation
import VideoToolbox

public enum VideoToolboxAV1SampleDecoderError: Error, Equatable, Sendable {
    case invalidIVF(String)
    case unsupported(String)
    case osStatus(operation: String, status: Int32)
    case missingFormatDescription
    case missingBlockBuffer
    case missingSampleBuffer
    case missingDecompressionSession
    case missingSampleBytes
    case missingDecodedImage
}

public struct VideoToolboxAV1SampleDecodeReport: Codable, Equatable, Sendable {
    public let samplePath: String
    public let sampleSHA256: String
    public let container: String
    public let codec: VideoCodec
    public let width: UInt16
    public let height: UInt16
    public let timescale: UInt32
    public let frameCount: UInt32
    public let framePayloadBytes: Int
    public let samplePayloadBytes: Int
    public let av1C: String
    public let hardwareDecodeSupported: Bool
    public let decodeStatus: String
    public let unsupportedReason: String?
    public let usingHardwareAcceleratedDecoder: Bool?
    public let decodedWidth: Int?
    public let decodedHeight: Int?
    public let decodedPixelFormat: String?
    public let decodeDurationNanoseconds: UInt64?

    public init(
        samplePath: String,
        sampleSHA256: String,
        container: String,
        codec: VideoCodec,
        width: UInt16,
        height: UInt16,
        timescale: UInt32,
        frameCount: UInt32,
        framePayloadBytes: Int,
        samplePayloadBytes: Int,
        av1C: String,
        hardwareDecodeSupported: Bool,
        decodeStatus: String,
        unsupportedReason: String?,
        usingHardwareAcceleratedDecoder: Bool?,
        decodedWidth: Int?,
        decodedHeight: Int?,
        decodedPixelFormat: String?,
        decodeDurationNanoseconds: UInt64?
    ) {
        self.samplePath = samplePath
        self.sampleSHA256 = sampleSHA256
        self.container = container
        self.codec = codec
        self.width = width
        self.height = height
        self.timescale = timescale
        self.frameCount = frameCount
        self.framePayloadBytes = framePayloadBytes
        self.samplePayloadBytes = samplePayloadBytes
        self.av1C = av1C
        self.hardwareDecodeSupported = hardwareDecodeSupported
        self.decodeStatus = decodeStatus
        self.unsupportedReason = unsupportedReason
        self.usingHardwareAcceleratedDecoder = usingHardwareAcceleratedDecoder
        self.decodedWidth = decodedWidth
        self.decodedHeight = decodedHeight
        self.decodedPixelFormat = decodedPixelFormat
        self.decodeDurationNanoseconds = decodeDurationNanoseconds
    }

    public func jsonData() throws -> Data {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return try encoder.encode(self)
    }
}

public enum VideoToolboxAV1SampleDecoder {
    public static func decodeIVFSample(
        at sampleURL: URL,
        samplePath: String? = nil
    ) throws -> VideoToolboxAV1SampleDecodeReport {
        let data = try Data(contentsOf: sampleURL)
        let sample = try IVFAV1Sample(data: data)
        let reportedPath = samplePath ?? sampleURL.path
        let sampleHash = sha256Hex(data)
        let hardwareSupported = VTIsHardwareDecodeSupported(kCMVideoCodecType_AV1)

        guard hardwareSupported else {
            return unsupportedReport(
                for: sample,
                samplePath: reportedPath,
                sampleSHA256: sampleHash,
                hardwareDecodeSupported: false,
                reason: "VTIsHardwareDecodeSupported(kCMVideoCodecType_AV1) returned false on this Mac."
            )
        }

        let decoded: DecodedFrame
        do {
            decoded = try VideoToolboxAV1DecodeCore.decode(sample: sample)
        } catch let error as VideoToolboxAV1SampleDecoderError {
            guard
                case let .osStatus(operation, status) = error,
                operation == "VTDecompressionSessionCreate"
            else {
                throw error
            }
            let reason = "\(operation) failed with OSStatus \(status) after " +
                "VTIsHardwareDecodeSupported(kCMVideoCodecType_AV1) returned true."
            return unsupportedReport(
                for: sample,
                samplePath: reportedPath,
                sampleSHA256: sampleHash,
                hardwareDecodeSupported: true,
                reason: reason
            )
        }

        return decodedReport(
            decoded,
            for: sample,
            samplePath: reportedPath,
            sampleSHA256: sampleHash
        )
    }

    public static func parseIVFSample(_ data: Data) throws -> ParsedIVFAV1Sample {
        let sample = try IVFAV1Sample(data: data)
        return ParsedIVFAV1Sample(
            width: sample.width,
            height: sample.height,
            timescale: sample.timescale,
            frameCount: sample.frameCount,
            framePayloadBytes: sample.framePayload.count,
            samplePayloadBytes: sample.videoToolboxSamplePayload.count,
            av1C: sample.av1C
        )
    }

    private static func sha256Hex(_ data: Data) -> String {
        SHA256.hash(data: data)
            .map { String(format: "%02x", $0) }
            .joined()
    }

    private static func hex(_ data: Data) -> String {
        data.map { String(format: "%02x", $0) }.joined()
    }

    private static func unsupportedReport(
        for sample: IVFAV1Sample,
        samplePath: String,
        sampleSHA256: String,
        hardwareDecodeSupported: Bool,
        reason: String
    ) -> VideoToolboxAV1SampleDecodeReport {
        VideoToolboxAV1SampleDecodeReport(
            samplePath: samplePath,
            sampleSHA256: sampleSHA256,
            container: "ivf",
            codec: .av1,
            width: sample.width,
            height: sample.height,
            timescale: sample.timescale,
            frameCount: sample.frameCount,
            framePayloadBytes: sample.framePayload.count,
            samplePayloadBytes: sample.videoToolboxSamplePayload.count,
            av1C: hex(sample.av1C),
            hardwareDecodeSupported: hardwareDecodeSupported,
            decodeStatus: "unsupported",
            unsupportedReason: reason,
            usingHardwareAcceleratedDecoder: nil,
            decodedWidth: nil,
            decodedHeight: nil,
            decodedPixelFormat: nil,
            decodeDurationNanoseconds: nil
        )
    }

    private static func decodedReport(
        _ decoded: DecodedFrame,
        for sample: IVFAV1Sample,
        samplePath: String,
        sampleSHA256: String
    ) -> VideoToolboxAV1SampleDecodeReport {
        VideoToolboxAV1SampleDecodeReport(
            samplePath: samplePath,
            sampleSHA256: sampleSHA256,
            container: "ivf",
            codec: .av1,
            width: sample.width,
            height: sample.height,
            timescale: sample.timescale,
            frameCount: sample.frameCount,
            framePayloadBytes: sample.framePayload.count,
            samplePayloadBytes: sample.videoToolboxSamplePayload.count,
            av1C: hex(sample.av1C),
            hardwareDecodeSupported: true,
            decodeStatus: "decoded",
            unsupportedReason: nil,
            usingHardwareAcceleratedDecoder: decoded.usingHardwareAcceleratedDecoder,
            decodedWidth: decoded.width,
            decodedHeight: decoded.height,
            decodedPixelFormat: decoded.pixelFormat,
            decodeDurationNanoseconds: decoded.durationNanoseconds
        )
    }
}

public struct ParsedIVFAV1Sample: Equatable, Sendable {
    public let width: UInt16
    public let height: UInt16
    public let timescale: UInt32
    public let frameCount: UInt32
    public let framePayloadBytes: Int
    public let samplePayloadBytes: Int
    public let av1C: Data
}
