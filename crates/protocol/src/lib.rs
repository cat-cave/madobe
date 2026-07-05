#![doc = "Protocol identity and wire metadata types for the madobe workspace."]
#![forbid(unsafe_code)]

mod cross_device;
mod product_quic;

pub use cross_device::{
    CrossDeviceEvidenceClaim, CrossDeviceResultArtifact, CrossDeviceResultArtifactKind,
    CrossDeviceResultValidationError, CrossDeviceVideoSmokeMetrics, CrossDeviceVideoSmokeResult,
};
pub use product_quic::{
    ProductQuicDownstreamClaim, ProductQuicEndpointRole, ProductQuicPayloadValidation,
    ProductQuicReceiverAck, ProductQuicResultArtifact, ProductQuicResultArtifactKind,
    ProductQuicResultValidationError, ProductQuicSmokeEndpoint, ProductQuicSmokeResult,
    ProductQuicTransport,
};

/// Human-readable product name shared by M0 components.
pub const PRODUCT_NAME: &str = "madobe";

/// Initial protocol version used by the M0 hello proof.
pub const PROTOCOL_VERSION: u16 = 1;

/// A small typed hello payload shared by the host daemon and CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MadobeHello {
    product: &'static str,
    protocol_version: u16,
}

impl MadobeHello {
    /// Returns the canonical hello payload for this workspace build.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            product: PRODUCT_NAME,
            protocol_version: PROTOCOL_VERSION,
        }
    }

    /// Returns the product identifier.
    #[must_use]
    pub const fn product(self) -> &'static str {
        self.product
    }

    /// Returns the protocol version.
    #[must_use]
    pub const fn protocol_version(self) -> u16 {
        self.protocol_version
    }

    /// Renders a deterministic identity segment for command output.
    #[must_use]
    pub fn identity(self) -> String {
        format!(
            "{} {} protocol={}",
            self.product,
            env!("CARGO_PKG_VERSION"),
            self.protocol_version
        )
    }
}

impl Default for MadobeHello {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the canonical product identity string.
#[must_use]
pub const fn product_identity() -> &'static str {
    PRODUCT_NAME
}

/// Encoded video codecs carried by the video lane metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VideoCodec {
    /// AV1 elementary stream payloads produced by the Linux encoder.
    Av1,
}

impl VideoCodec {
    /// Returns the stable lower-case wire token for this codec.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Av1 => "av1",
        }
    }
}

/// Hash algorithms used to identify encoded frame payload bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PayloadHashAlgorithm {
    /// SHA-256 encoded as 64 lowercase hexadecimal characters.
    Sha256,
}

impl PayloadHashAlgorithm {
    /// Returns the stable lower-case wire token for this hash algorithm.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
        }
    }
}

/// Cryptographic hash of the encoded frame payload bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadHash {
    /// Algorithm used to produce `value`.
    pub algorithm: PayloadHashAlgorithm,
    /// Lowercase hexadecimal digest for the payload bytes.
    pub value: String,
}

/// Wire metadata for one encoded video frame.
///
/// This type intentionally describes encoded bytes and timing only. It does not
/// claim capture, transport, decode, render, or latency behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedVideoFrameMetadata {
    /// Monotonic sender-local frame identifier.
    pub frame_id: u64,
    /// Codec for the encoded payload.
    pub codec: VideoCodec,
    /// Encoded frame width in pixels.
    pub width: u32,
    /// Encoded frame height in pixels.
    pub height: u32,
    /// Sender-local capture timestamp in nanoseconds.
    pub capture_timestamp_ns: u64,
    /// Sender-local encode completion timestamp in nanoseconds.
    pub encode_timestamp_ns: u64,
    /// True when this frame can initialize decoder state.
    pub keyframe: bool,
    /// Byte count of the encoded payload.
    pub payload_bytes: u32,
    /// Hash of the encoded payload bytes.
    pub payload_hash: PayloadHash,
}

#[cfg(test)]
mod tests {
    use super::{
        EncodedVideoFrameMetadata, MadobeHello, PRODUCT_NAME, PROTOCOL_VERSION,
        PayloadHashAlgorithm, VideoCodec, product_identity,
    };

    const GOLDEN_FRAME_JSON: &str = include_str!("../fixtures/encoded-video-frame-av1.json");
    const GOLDEN_PAYLOAD_HASH: &str =
        "4808d39bb0065087388612224cfda59f52f0278772f390065aa5e743f7bc0667";
    const EXPECTED_GOLDEN_FRAME_JSON: &str = concat!(
        "{\n",
        "  \"frameId\": 42,\n",
        "  \"codec\": \"av1\",\n",
        "  \"width\": 2560,\n",
        "  \"height\": 1440,\n",
        "  \"captureTimestampNs\": 1720000000000000000,\n",
        "  \"encodeTimestampNs\": 1720000000004166667,\n",
        "  \"keyframe\": true,\n",
        "  \"payloadBytes\": 38,\n",
        "  \"payloadHash\": {\n",
        "    \"algorithm\": \"sha256\",\n",
        "    \"value\": \"4808d39bb0065087388612224cfda59f52f0278772f390065aa5e743f7bc0667\"\n",
        "  }\n",
        "}\n",
    );

    #[test]
    fn hello_carries_stable_identity() {
        let hello = MadobeHello::new();

        assert_eq!(hello.product(), PRODUCT_NAME);
        assert_eq!(hello.protocol_version(), PROTOCOL_VERSION);
        assert_eq!(hello.identity(), "madobe 0.1.0 protocol=1");
        assert_eq!(product_identity(), "madobe");
    }

    #[test]
    fn golden_video_frame_fixture_covers_expected_metadata() {
        let frame = golden_frame_metadata();

        assert_eq!(frame.frame_id, 42);
        assert_eq!(frame.codec, VideoCodec::Av1);
        assert_eq!(frame.codec.wire_name(), "av1");
        assert_eq!(frame.width, 2560);
        assert_eq!(frame.height, 1440);
        assert_eq!(frame.capture_timestamp_ns, 1_720_000_000_000_000_000);
        assert_eq!(frame.encode_timestamp_ns, 1_720_000_000_004_166_667);
        assert!(frame.keyframe);
        assert_eq!(frame.payload_bytes, 38);
        assert_eq!(frame.payload_hash.algorithm, PayloadHashAlgorithm::Sha256);
        assert_eq!(frame.payload_hash.algorithm.wire_name(), "sha256");
        assert_eq!(frame.payload_hash.value.as_str(), GOLDEN_PAYLOAD_HASH);
        assert_eq!(GOLDEN_FRAME_JSON, EXPECTED_GOLDEN_FRAME_JSON);
    }

    #[test]
    fn golden_video_frame_fixture_round_trips_semantically() {
        let frame = golden_frame_metadata();

        assert_eq!(encode_golden_fixture(&frame), GOLDEN_FRAME_JSON);
    }

    fn golden_frame_metadata() -> EncodedVideoFrameMetadata {
        EncodedVideoFrameMetadata {
            frame_id: 42,
            codec: VideoCodec::Av1,
            width: 2560,
            height: 1440,
            capture_timestamp_ns: 1_720_000_000_000_000_000,
            encode_timestamp_ns: 1_720_000_000_004_166_667,
            keyframe: true,
            payload_bytes: 38,
            payload_hash: super::PayloadHash {
                algorithm: PayloadHashAlgorithm::Sha256,
                value: GOLDEN_PAYLOAD_HASH.to_owned(),
            },
        }
    }

    fn encode_golden_fixture(frame: &EncodedVideoFrameMetadata) -> String {
        let frame_id = frame.frame_id;
        let codec = frame.codec.wire_name();
        let width = frame.width;
        let height = frame.height;
        let capture_timestamp_ns = frame.capture_timestamp_ns;
        let encode_timestamp_ns = frame.encode_timestamp_ns;
        let keyframe = frame.keyframe;
        let payload_bytes = frame.payload_bytes;
        let payload_hash_algorithm = frame.payload_hash.algorithm.wire_name();
        let payload_hash_value = frame.payload_hash.value.as_str();

        format!(
            concat!(
                "{{\n",
                "  \"frameId\": {frame_id},\n",
                "  \"codec\": \"{codec}\",\n",
                "  \"width\": {width},\n",
                "  \"height\": {height},\n",
                "  \"captureTimestampNs\": {capture_timestamp_ns},\n",
                "  \"encodeTimestampNs\": {encode_timestamp_ns},\n",
                "  \"keyframe\": {keyframe},\n",
                "  \"payloadBytes\": {payload_bytes},\n",
                "  \"payloadHash\": {{\n",
                "    \"algorithm\": \"{payload_hash_algorithm}\",\n",
                "    \"value\": \"{payload_hash_value}\"\n",
                "  }}\n",
                "}}\n",
            ),
            frame_id = frame_id,
            codec = codec,
            width = width,
            height = height,
            capture_timestamp_ns = capture_timestamp_ns,
            encode_timestamp_ns = encode_timestamp_ns,
            keyframe = keyframe,
            payload_bytes = payload_bytes,
            payload_hash_algorithm = payload_hash_algorithm,
            payload_hash_value = payload_hash_value
        )
    }
}
