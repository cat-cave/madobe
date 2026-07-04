import CoreMedia
import CoreVideo
import Foundation
import VideoToolbox

struct DecodedFrame {
    let width: Int
    let height: Int
    let pixelFormat: String
    let usingHardwareAcceleratedDecoder: Bool?
    let durationNanoseconds: UInt64
}

private final class DecodeOutput: @unchecked Sendable {
    var status: OSStatus = noErr
    var image: CVImageBuffer?
}

enum VideoToolboxAV1DecodeCore {
    static func decode(sample: IVFAV1Sample) throws -> DecodedFrame {
        let formatDescription = try makeFormatDescription(for: sample)
        let sampleBuffer = try makeSampleBuffer(
            for: sample,
            formatDescription: formatDescription
        )
        let session = try makeDecompressionSession(
            formatDescription: formatDescription
        )

        return try decodeFrame(
            sampleBuffer: sampleBuffer,
            session: session
        )
    }

    private static func makeFormatDescription(
        for sample: IVFAV1Sample
    ) throws -> CMVideoFormatDescription {
        var formatDescription: CMVideoFormatDescription?
        try check(
            "CMVideoFormatDescriptionCreate",
            CMVideoFormatDescriptionCreate(
                allocator: kCFAllocatorDefault,
                codecType: kCMVideoCodecType_AV1,
                width: Int32(sample.width),
                height: Int32(sample.height),
                extensions: sample.formatDescriptionExtensions,
                formatDescriptionOut: &formatDescription
            )
        )

        guard let formatDescription else {
            throw VideoToolboxAV1SampleDecoderError.missingFormatDescription
        }
        return formatDescription
    }

    private static func makeSampleBuffer(
        for sample: IVFAV1Sample,
        formatDescription: CMVideoFormatDescription
    ) throws -> CMSampleBuffer {
        let blockBuffer = try makeBlockBuffer(
            payload: sample.videoToolboxSamplePayload
        )
        var timing = CMSampleTimingInfo(
            duration: CMTime(value: 1, timescale: CMTimeScale(sample.timescale)),
            presentationTimeStamp: .zero,
            decodeTimeStamp: .invalid
        )
        var sampleSize = sample.videoToolboxSamplePayload.count
        var sampleBuffer: CMSampleBuffer?
        try check(
            "CMSampleBufferCreateReady",
            CMSampleBufferCreateReady(
                allocator: kCFAllocatorDefault,
                dataBuffer: blockBuffer,
                formatDescription: formatDescription,
                sampleCount: 1,
                sampleTimingEntryCount: 1,
                sampleTimingArray: &timing,
                sampleSizeEntryCount: 1,
                sampleSizeArray: &sampleSize,
                sampleBufferOut: &sampleBuffer
            )
        )

        guard let sampleBuffer else {
            throw VideoToolboxAV1SampleDecoderError.missingSampleBuffer
        }
        return sampleBuffer
    }

    private static func makeBlockBuffer(payload: Data) throws -> CMBlockBuffer {
        var blockBuffer: CMBlockBuffer?
        try check(
            "CMBlockBufferCreateWithMemoryBlock",
            CMBlockBufferCreateWithMemoryBlock(
                allocator: kCFAllocatorDefault,
                memoryBlock: nil,
                blockLength: payload.count,
                blockAllocator: kCFAllocatorDefault,
                customBlockSource: nil,
                offsetToData: 0,
                dataLength: payload.count,
                flags: 0,
                blockBufferOut: &blockBuffer
            )
        )

        guard let blockBuffer else {
            throw VideoToolboxAV1SampleDecoderError.missingBlockBuffer
        }
        try payload.withUnsafeBytes { bytes in
            guard let baseAddress = bytes.baseAddress else {
                throw VideoToolboxAV1SampleDecoderError.missingSampleBytes
            }
            try check(
                "CMBlockBufferReplaceDataBytes",
                CMBlockBufferReplaceDataBytes(
                    with: baseAddress,
                    blockBuffer: blockBuffer,
                    offsetIntoDestination: 0,
                    dataLength: payload.count
                )
            )
        }
        return blockBuffer
    }

    private static func makeDecompressionSession(
        formatDescription: CMVideoFormatDescription
    ) throws -> VTDecompressionSession {
        let requireHardwareDecode: CFBoolean = kCFBooleanTrue
        let decoderSpecification: [CFString: Any] = [
            kVTVideoDecoderSpecification_RequireHardwareAcceleratedVideoDecoder: requireHardwareDecode,
        ]
        let imageAttributes: [CFString: Any] = [
            kCVPixelBufferPixelFormatTypeKey: kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange,
        ]
        var session: VTDecompressionSession?
        try check(
            "VTDecompressionSessionCreate",
            VTDecompressionSessionCreate(
                allocator: kCFAllocatorDefault,
                formatDescription: formatDescription,
                decoderSpecification: decoderSpecification as CFDictionary,
                imageBufferAttributes: imageAttributes as CFDictionary,
                outputCallback: nil,
                decompressionSessionOut: &session
            )
        )

        guard let session else {
            throw VideoToolboxAV1SampleDecoderError.missingDecompressionSession
        }
        return session
    }

    private static func decodeFrame(
        sampleBuffer: CMSampleBuffer,
        session: VTDecompressionSession
    ) throws -> DecodedFrame {
        let start = DispatchTime.now().uptimeNanoseconds
        let output = DecodeOutput()
        var infoFlags = VTDecodeInfoFlags()
        try check(
            "VTDecompressionSessionDecodeFrame",
            VTDecompressionSessionDecodeFrame(
                session,
                sampleBuffer: sampleBuffer,
                flags: [],
                infoFlagsOut: &infoFlags
            ) { status, _, imageBuffer, _, _ in
                output.status = status
                output.image = imageBuffer
            }
        )
        let duration = DispatchTime.now().uptimeNanoseconds - start
        try check("VideoToolbox decode callback", output.status)

        guard let image = output.image else {
            throw VideoToolboxAV1SampleDecoderError.missingDecodedImage
        }

        return DecodedFrame(
            width: CVPixelBufferGetWidth(image),
            height: CVPixelBufferGetHeight(image),
            pixelFormat: fourCharacterCodeString(CVPixelBufferGetPixelFormatType(image)),
            usingHardwareAcceleratedDecoder: copyUsingHardwareDecoder(session),
            durationNanoseconds: duration
        )
    }

    private static func check(_ operation: String, _ status: OSStatus) throws {
        guard status == noErr else {
            throw VideoToolboxAV1SampleDecoderError.osStatus(
                operation: operation,
                status: status
            )
        }
    }

    private static func copyUsingHardwareDecoder(_ session: VTDecompressionSession) -> Bool? {
        var value: CFTypeRef?
        let status = withUnsafeMutablePointer(to: &value) { pointer in
            VTSessionCopyProperty(
                session,
                key: kVTDecompressionPropertyKey_UsingHardwareAcceleratedVideoDecoder,
                allocator: kCFAllocatorDefault,
                valueOut: UnsafeMutableRawPointer(pointer)
            )
        }
        guard status == noErr else {
            return nil
        }
        return (value as? Bool)
    }

    private static func fourCharacterCodeString(_ code: OSType) -> String {
        let bytes = [
            UInt8((code >> 24) & 0xFF),
            UInt8((code >> 16) & 0xFF),
            UInt8((code >> 8) & 0xFF),
            UInt8(code & 0xFF),
        ]
        return String(bytes: bytes, encoding: .ascii) ?? String(code)
    }
}
