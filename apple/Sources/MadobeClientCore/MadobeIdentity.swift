import Foundation

public struct MadobeIdentity: Equatable, Sendable {
    public let product: String
    public let protocolVersion: Int
    public let eventName: String
    public let timestampMillis: Int

    public init(
        product: String = "madobe",
        protocolVersion: Int = 1,
        eventName: String = "madobe.bootstrap",
        timestampMillis: Int = 0
    ) {
        self.product = product
        self.protocolVersion = protocolVersion
        self.eventName = eventName
        self.timestampMillis = timestampMillis
    }

    public var statusLine: String {
        "\(product) 0.1.0 protocol=\(protocolVersion) event=\(eventName) ts=\(timestampMillis) status=ok"
    }
}

public enum MadobeClientCore {
    public static func identity() -> MadobeIdentity {
        MadobeIdentity()
    }
}
