import CoreMedia
import Foundation
import VideoToolbox

public struct VideoToolboxCodecCapability: Codable, Equatable, Sendable {
    public let codec: VideoCodec
    public let coreMediaFourCharacterCode: String
    public let hardwareDecodeSupported: Bool

    public init(
        codec: VideoCodec,
        coreMediaFourCharacterCode: String,
        hardwareDecodeSupported: Bool
    ) {
        self.codec = codec
        self.coreMediaFourCharacterCode = coreMediaFourCharacterCode
        self.hardwareDecodeSupported = hardwareDecodeSupported
    }
}

public struct VideoToolboxCapabilityReport: Codable, Equatable, Sendable {
    public let operatingSystemVersion: String
    public let codecs: [VideoToolboxCodecCapability]

    public init(
        operatingSystemVersion: String,
        codecs: [VideoToolboxCodecCapability]
    ) {
        self.operatingSystemVersion = operatingSystemVersion
        self.codecs = codecs
    }
}

public enum VideoToolboxCapabilityProbe {
    public static func report(
        operatingSystemVersion: String = ProcessInfo.processInfo.operatingSystemVersionString
    ) -> VideoToolboxCapabilityReport {
        VideoToolboxCapabilityReport(
            operatingSystemVersion: operatingSystemVersion,
            codecs: [
                VideoToolboxCodecCapability(
                    codec: .av1,
                    coreMediaFourCharacterCode: fourCharacterCodeString(kCMVideoCodecType_AV1),
                    hardwareDecodeSupported: VTIsHardwareDecodeSupported(kCMVideoCodecType_AV1)
                ),
            ]
        )
    }

    public static func jsonData(
        for report: VideoToolboxCapabilityReport = report()
    ) throws -> Data {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return try encoder.encode(report)
    }

    private static func fourCharacterCodeString(_ code: FourCharCode) -> String {
        let bytes = [
            UInt8((code >> 24) & 0xFF),
            UInt8((code >> 16) & 0xFF),
            UInt8((code >> 8) & 0xFF),
            UInt8(code & 0xFF),
        ]
        return String(bytes: bytes, encoding: .ascii) ?? String(code)
    }
}
