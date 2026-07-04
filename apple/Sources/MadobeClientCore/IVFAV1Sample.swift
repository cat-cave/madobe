import CoreMedia
import Foundation

struct IVFAV1Sample {
    let width: UInt16
    let height: UInt16
    let timescale: UInt32
    let frameCount: UInt32
    let framePayload: Data
    let videoToolboxSamplePayload: Data
    let av1C: Data

    init(data: Data) throws {
        guard data.count >= 44 else {
            throw VideoToolboxAV1SampleDecoderError
                .invalidIVF(
                    "IVF sample is shorter than the 32-byte file header plus one 12-byte frame header."
                )
        }
        guard String(bytes: data[0 ..< 4], encoding: .ascii) == "DKIF" else {
            throw VideoToolboxAV1SampleDecoderError.invalidIVF("IVF header magic is not DKIF.")
        }
        guard String(bytes: data[8 ..< 12], encoding: .ascii) == "AV01" else {
            throw VideoToolboxAV1SampleDecoderError.invalidIVF("IVF fourcc is not AV01.")
        }

        width = data.uint16LE(at: 12)
        height = data.uint16LE(at: 14)
        timescale = data.uint32LE(at: 16)
        frameCount = data.uint32LE(at: 24)

        let frameSize = Int(data.uint32LE(at: 32))
        guard frameSize > 0 else {
            throw VideoToolboxAV1SampleDecoderError
                .invalidIVF("First IVF frame has zero payload bytes.")
        }
        guard data.count >= 44 + frameSize else {
            throw VideoToolboxAV1SampleDecoderError
                .invalidIVF("First IVF frame payload is truncated.")
        }

        framePayload = data.subdata(in: 44 ..< 44 + frameSize)
        if framePayload.starts(with: [0x12, 0x00]) {
            videoToolboxSamplePayload = framePayload.dropFirst(2)
        } else {
            videoToolboxSamplePayload = framePayload
        }

        guard videoToolboxSamplePayload.count >= 12 else {
            throw VideoToolboxAV1SampleDecoderError
                .invalidIVF("AV1 sample payload is too short to contain the sequence header OBU.")
        }
        av1C = Data([0x81, 0x00, 0x0C, 0x00]) + videoToolboxSamplePayload.prefix(12)
    }

    var formatDescriptionExtensions: CFDictionary {
        [
            kCMFormatDescriptionExtension_SampleDescriptionExtensionAtoms: [
                "av1C": av1C as CFData,
            ] as CFDictionary,
        ] as CFDictionary
    }
}

private extension Data {
    func uint16LE(at offset: Int) -> UInt16 {
        UInt16(self[offset]) | UInt16(self[offset + 1]) << 8
    }

    func uint32LE(at offset: Int) -> UInt32 {
        UInt32(self[offset])
            | UInt32(self[offset + 1]) << 8
            | UInt32(self[offset + 2]) << 16
            | UInt32(self[offset + 3]) << 24
    }
}
