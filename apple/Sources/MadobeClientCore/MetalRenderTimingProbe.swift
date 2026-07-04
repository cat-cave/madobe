import Dispatch
import Foundation
import Metal

public enum MetalRenderTimingProbeError: Error, Equatable, Sendable {
    case metalDeviceUnavailable
    case commandQueueUnavailable
    case textureUnavailable
    case commandBufferUnavailable
    case renderCommandEncoderUnavailable
    case commandBufferFailed(String)
}

public struct MetalTestPatternColor: Codable, Equatable, Sendable {
    public let red: Double
    public let green: Double
    public let blue: Double
    public let alpha: Double

    public init(red: Double, green: Double, blue: Double, alpha: Double) {
        self.red = red
        self.green = green
        self.blue = blue
        self.alpha = alpha
    }

    public static let validationRed = MetalTestPatternColor(
        red: 1,
        green: 0,
        blue: 0,
        alpha: 1
    )

    var clearColor: MTLClearColor {
        MTLClearColor(red: red, green: green, blue: blue, alpha: alpha)
    }

    var bgra8Bytes: [UInt8] {
        [
            Self.unorm8(blue),
            Self.unorm8(green),
            Self.unorm8(red),
            Self.unorm8(alpha),
        ]
    }

    private static func unorm8(_ value: Double) -> UInt8 {
        UInt8((min(max(value, 0), 1) * 255).rounded())
    }
}

public struct MetalRenderTimingReport: Codable, Equatable, Sendable {
    public let renderer: String
    public let deviceName: String
    public let width: Int
    public let height: Int
    public let pixelFormat: String
    public let renderTargetKind: String
    public let clearColor: MetalTestPatternColor
    public let sampledPixelBGRA8: [UInt8]
    public let commandBufferStatus: String
    public let offscreenTestPatternRendered: Bool
    public let displayPresented: Bool
    public let renderDurationNanoseconds: UInt64

    public init(
        renderer: String,
        deviceName: String,
        width: Int,
        height: Int,
        pixelFormat: String,
        renderTargetKind: String,
        clearColor: MetalTestPatternColor,
        sampledPixelBGRA8: [UInt8],
        commandBufferStatus: String,
        offscreenTestPatternRendered: Bool,
        displayPresented: Bool,
        renderDurationNanoseconds: UInt64
    ) {
        self.renderer = renderer
        self.deviceName = deviceName
        self.width = width
        self.height = height
        self.pixelFormat = pixelFormat
        self.renderTargetKind = renderTargetKind
        self.clearColor = clearColor
        self.sampledPixelBGRA8 = sampledPixelBGRA8
        self.commandBufferStatus = commandBufferStatus
        self.offscreenTestPatternRendered = offscreenTestPatternRendered
        self.displayPresented = displayPresented
        self.renderDurationNanoseconds = renderDurationNanoseconds
    }

    public func jsonData() throws -> Data {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        return try encoder.encode(self)
    }
}

public enum MetalRenderTimingProbe {
    public static var isSupported: Bool {
        MTLCreateSystemDefaultDevice() != nil
    }

    public static func renderTestPatternFrame(
        width: Int = 320,
        height: Int = 180,
        clearColor: MetalTestPatternColor = .validationRed
    ) throws -> MetalRenderTimingReport {
        guard let device = MTLCreateSystemDefaultDevice() else {
            throw MetalRenderTimingProbeError.metalDeviceUnavailable
        }
        guard let commandQueue = device.makeCommandQueue() else {
            throw MetalRenderTimingProbeError.commandQueueUnavailable
        }
        let texture = try makeRenderTarget(device: device, width: width, height: height)
        guard let commandBuffer = commandQueue.makeCommandBuffer() else {
            throw MetalRenderTimingProbeError.commandBufferUnavailable
        }

        let start = DispatchTime.now().uptimeNanoseconds
        try encodeClearPass(
            commandBuffer: commandBuffer,
            texture: texture,
            clearColor: clearColor
        )
        commandBuffer.commit()
        commandBuffer.waitUntilCompleted()
        let duration = DispatchTime.now().uptimeNanoseconds - start

        try checkCompleted(commandBuffer)
        return MetalRenderTimingReport(
            renderer: "metal-clear-test-pattern",
            deviceName: device.name,
            width: width,
            height: height,
            pixelFormat: "bgra8Unorm",
            renderTargetKind: "offscreen-texture",
            clearColor: clearColor,
            sampledPixelBGRA8: sampleFirstPixel(texture),
            commandBufferStatus: commandBuffer.status.reportName,
            offscreenTestPatternRendered: true,
            displayPresented: false,
            renderDurationNanoseconds: duration
        )
    }

    private static func makeRenderTarget(
        device: MTLDevice,
        width: Int,
        height: Int
    ) throws -> MTLTexture {
        let descriptor = MTLTextureDescriptor.texture2DDescriptor(
            pixelFormat: .bgra8Unorm,
            width: width,
            height: height,
            mipmapped: false
        )
        descriptor.storageMode = .shared
        descriptor.usage = [.renderTarget, .shaderRead]
        guard let texture = device.makeTexture(descriptor: descriptor) else {
            throw MetalRenderTimingProbeError.textureUnavailable
        }
        return texture
    }

    private static func encodeClearPass(
        commandBuffer: MTLCommandBuffer,
        texture: MTLTexture,
        clearColor: MetalTestPatternColor
    ) throws {
        let descriptor = MTLRenderPassDescriptor()
        let attachment = descriptor.colorAttachments[0]
        attachment?.texture = texture
        attachment?.loadAction = .clear
        attachment?.clearColor = clearColor.clearColor
        attachment?.storeAction = .store

        guard let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: descriptor) else {
            throw MetalRenderTimingProbeError.renderCommandEncoderUnavailable
        }
        encoder.endEncoding()
    }

    private static func checkCompleted(_ commandBuffer: MTLCommandBuffer) throws {
        guard commandBuffer.status == .completed else {
            let message = commandBuffer.error?.localizedDescription ?? commandBuffer.status
                .reportName
            throw MetalRenderTimingProbeError.commandBufferFailed(message)
        }
    }

    private static func sampleFirstPixel(_ texture: MTLTexture) -> [UInt8] {
        var pixel = [UInt8](repeating: 0, count: 4)
        pixel.withUnsafeMutableBytes { buffer in
            guard let baseAddress = buffer.baseAddress else {
                return
            }
            texture.getBytes(
                baseAddress,
                bytesPerRow: 4,
                from: MTLRegionMake2D(0, 0, 1, 1),
                mipmapLevel: 0
            )
        }
        return pixel
    }
}

private extension MTLCommandBufferStatus {
    var reportName: String {
        switch self {
        case .notEnqueued:
            return "notEnqueued"
        case .enqueued:
            return "enqueued"
        case .committed:
            return "committed"
        case .scheduled:
            return "scheduled"
        case .completed:
            return "completed"
        case .error:
            return "error"
        @unknown default:
            return "unknown"
        }
    }
}
