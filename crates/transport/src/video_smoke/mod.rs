//! Dependency-free TCP AV1 sample smoke harness.
//!
//! This module sends the checked-in IVF sample as opaque bytes over TCP and
//! validates metadata, byte count, and SHA-256 on receive. It is not the
//! product QUIC transport and does not decode, render, present, or measure
//! latency.

mod artifacts;
mod cli;
mod sha256;
mod wire;

use std::error::Error;
use std::fmt::{self, Display};
use std::fs;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};

use madobe_protocol::{EncodedVideoFrameMetadata, PayloadHash, PayloadHashAlgorithm, VideoCodec};

use crate::{TransportError, VideoSample};

pub use artifacts::{SmokeArtifactSet, SmokeSide};
pub use cli::run_cli;

/// Repository-relative path to the checked-in AV1 IVF sample.
pub const CHECKED_IN_AV1_SAMPLE: &str = "evidence/m2-nvenc-encode-sample/sample-av1.ivf";

/// Expected SHA-256 of the checked-in AV1 IVF sample.
pub const CHECKED_IN_AV1_SHA256: &str =
    "51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875";

/// Stable node id recorded in smoke artifacts.
pub const NODE_ID: &str = "m3-lan-video-smoke-harness";

/// One loaded AV1 sample ready for the LAN smoke harness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmokeSample {
    /// Metadata sent before the payload bytes.
    pub metadata: EncodedVideoFrameMetadata,
    /// Opaque IVF sample bytes sent as the frame payload.
    pub payload: Vec<u8>,
    /// Source path used to load this sample.
    pub source_path: PathBuf,
}

impl SmokeSample {
    /// Loads the checked-in AV1 IVF sample from disk.
    ///
    /// # Errors
    ///
    /// Returns an error when the file cannot be read, the IVF header is not
    /// the expected AV1 shape, or the file hash is not the checked-in sample
    /// hash.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, SmokeError> {
        let path = path.as_ref();
        let payload = fs::read(path).map_err(|error| SmokeError::io("read sample", &error))?;
        let header = IvfHeader::parse(&payload)?;
        let hash = sha256::digest_hex(&payload);

        if hash != CHECKED_IN_AV1_SHA256 {
            return Err(SmokeError::invalid_sample(format!(
                "sample sha256 {hash} did not match checked-in hash {CHECKED_IN_AV1_SHA256}"
            )));
        }

        let payload_bytes = u32::try_from(payload.len()).map_err(|_| {
            SmokeError::invalid_sample(format!(
                "sample has {} bytes and exceeds u32",
                payload.len()
            ))
        })?;
        let metadata = EncodedVideoFrameMetadata {
            frame_id: 1,
            codec: VideoCodec::Av1,
            width: u32::from(header.width),
            height: u32::from(header.height),
            capture_timestamp_ns: 0,
            encode_timestamp_ns: 0,
            keyframe: true,
            payload_bytes,
            payload_hash: PayloadHash {
                algorithm: PayloadHashAlgorithm::Sha256,
                value: hash,
            },
        };

        Ok(Self {
            metadata,
            payload,
            source_path: path.to_path_buf(),
        })
    }

    fn into_video_sample(self) -> Result<VideoSample, SmokeError> {
        VideoSample::new(self.metadata, self.payload).map_err(|error| SmokeError::transport(&error))
    }
}

/// Summary returned by a sender run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmokeSendSummary {
    /// Remote peer address used by the sender.
    pub remote_addr: SocketAddr,
    /// Frame metadata sent on the TCP connection.
    pub metadata: EncodedVideoFrameMetadata,
    /// Number of payload bytes sent.
    pub payload_bytes: usize,
}

/// Validation summary returned by a receiver run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmokeReceiveSummary {
    /// Local listening address used by the receiver.
    pub local_addr: SocketAddr,
    /// Sender peer address accepted by the receiver.
    pub peer_addr: SocketAddr,
    /// Frame metadata received on the TCP connection.
    pub metadata: EncodedVideoFrameMetadata,
    /// Number of payload bytes received.
    pub payload_bytes: usize,
    /// SHA-256 computed over the received payload bytes.
    pub payload_sha256: String,
    /// Whether all receiver validation checks passed.
    pub passed: bool,
}

/// Sends the checked-in AV1 sample to a TCP receiver.
///
/// # Errors
///
/// Returns an error when the sample cannot be loaded, the peer cannot be
/// reached, wire encoding fails, or artifact files cannot be written.
pub fn send_checked_av1_sample(
    addr: impl ToSocketAddrs,
    sample_path: impl AsRef<Path>,
    artifact_dir: Option<&Path>,
) -> Result<SmokeSendSummary, SmokeError> {
    let remote_addr = resolve_one(addr)?;
    let artifacts = SmokeArtifactSet::new(artifact_dir);
    let sample = SmokeSample::load(sample_path)?;
    let metadata = sample.metadata.clone();
    let payload_bytes = sample.payload.len();
    let mut stream =
        TcpStream::connect(remote_addr).map_err(|error| SmokeError::io("connect", &error))?;
    wire::write_sample(&mut stream, &sample.into_video_sample()?)?;

    let summary = SmokeSendSummary {
        remote_addr,
        metadata,
        payload_bytes,
    };
    artifacts.write_sender(&summary)?;

    Ok(summary)
}

/// Receives and validates one checked-in AV1 sample on a TCP listener.
///
/// # Errors
///
/// Returns an error when accepting, reading, validating, or artifact writing
/// fails.
pub fn receive_checked_av1_sample(
    listener: &TcpListener,
    artifact_dir: Option<&Path>,
) -> Result<SmokeReceiveSummary, SmokeError> {
    let local_addr = listener
        .local_addr()
        .map_err(|error| SmokeError::io("inspect listener", &error))?;
    let artifacts = SmokeArtifactSet::new(artifact_dir);
    artifacts.write_receiver_listening(local_addr)?;
    let (mut stream, peer_addr) = listener
        .accept()
        .map_err(|error| SmokeError::io("accept", &error))?;
    let sample = wire::read_sample(&mut stream)?;
    let payload_sha256 = sha256::digest_hex(&sample.payload);
    validate_received_sample(&sample, &payload_sha256)?;
    let summary = SmokeReceiveSummary {
        local_addr,
        peer_addr,
        metadata: sample.metadata,
        payload_bytes: sample.payload.len(),
        payload_sha256,
        passed: true,
    };
    artifacts.write_receiver(&summary)?;

    Ok(summary)
}

/// Binds a TCP receiver, accepts one sample, and validates it.
///
/// # Errors
///
/// Returns an error when binding or receiving fails.
pub fn receive_checked_av1_sample_on(
    bind_addr: impl ToSocketAddrs,
    artifact_dir: Option<&Path>,
) -> Result<SmokeReceiveSummary, SmokeError> {
    let listener = TcpListener::bind(bind_addr).map_err(|error| SmokeError::io("bind", &error))?;
    receive_checked_av1_sample(&listener, artifact_dir)
}

fn resolve_one(addr: impl ToSocketAddrs) -> Result<SocketAddr, SmokeError> {
    addr.to_socket_addrs()
        .map_err(|error| SmokeError::io("resolve address", &error))?
        .next()
        .ok_or_else(|| SmokeError::usage("address resolved to no socket addresses"))
}

fn validate_received_sample(sample: &VideoSample, payload_sha256: &str) -> Result<(), SmokeError> {
    if sample.metadata.codec != VideoCodec::Av1 {
        return Err(SmokeError::validation("metadata codec was not av1"));
    }
    if sample.metadata.width != 160 || sample.metadata.height != 90 {
        return Err(SmokeError::validation(
            "metadata dimensions were not 160x90",
        ));
    }
    if sample.metadata.frame_id != 1 {
        return Err(SmokeError::validation("metadata frame id was not 1"));
    }
    if sample.metadata.capture_timestamp_ns != 0 || sample.metadata.encode_timestamp_ns != 0 {
        return Err(SmokeError::validation(
            "metadata timestamps were not deterministic zero values",
        ));
    }
    if !sample.metadata.keyframe {
        return Err(SmokeError::validation(
            "metadata did not mark the sample as a keyframe",
        ));
    }
    if sample.metadata.payload_bytes != 84 || sample.payload.len() != 84 {
        return Err(SmokeError::validation("payload byte count was not 84"));
    }
    if sample.metadata.payload_hash.algorithm != PayloadHashAlgorithm::Sha256 {
        return Err(SmokeError::validation(
            "payload hash algorithm was not sha256",
        ));
    }
    if sample.metadata.payload_hash.value != CHECKED_IN_AV1_SHA256 {
        return Err(SmokeError::validation(
            "metadata hash did not match checked-in sample",
        ));
    }
    if payload_sha256 != sample.metadata.payload_hash.value {
        return Err(SmokeError::validation(
            "payload bytes did not match metadata sha256",
        ));
    }

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IvfHeader {
    width: u16,
    height: u16,
}

impl IvfHeader {
    fn parse(payload: &[u8]) -> Result<Self, SmokeError> {
        if payload.len() < 32 {
            return Err(SmokeError::invalid_sample(
                "sample is shorter than an IVF header",
            ));
        }
        if &payload[0..4] != b"DKIF" {
            return Err(SmokeError::invalid_sample(
                "sample header magic was not DKIF",
            ));
        }
        if &payload[8..12] != b"AV01" {
            return Err(SmokeError::invalid_sample(
                "sample IVF codec tag was not AV01",
            ));
        }

        Ok(Self {
            width: u16::from_le_bytes([payload[12], payload[13]]),
            height: u16::from_le_bytes([payload[14], payload[15]]),
        })
    }
}

/// Error returned by the AV1 LAN smoke harness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmokeError {
    kind: SmokeErrorKind,
}

impl SmokeError {
    /// Returns the machine-readable error kind.
    #[must_use]
    pub const fn kind(&self) -> &SmokeErrorKind {
        &self.kind
    }

    fn io(operation: &'static str, error: &io::Error) -> Self {
        Self {
            kind: SmokeErrorKind::Io {
                operation,
                message: error.to_string(),
            },
        }
    }

    fn invalid_sample(message: impl Into<String>) -> Self {
        Self {
            kind: SmokeErrorKind::InvalidSample {
                message: message.into(),
            },
        }
    }

    fn transport(error: &TransportError) -> Self {
        Self {
            kind: SmokeErrorKind::Transport {
                message: error.to_string(),
            },
        }
    }

    fn validation(message: impl Into<String>) -> Self {
        Self {
            kind: SmokeErrorKind::Validation {
                message: message.into(),
            },
        }
    }

    fn wire(message: impl Into<String>) -> Self {
        Self {
            kind: SmokeErrorKind::Wire {
                message: message.into(),
            },
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self {
            kind: SmokeErrorKind::Usage {
                message: message.into(),
            },
        }
    }
}

impl Display for SmokeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            SmokeErrorKind::Io { operation, message } => {
                write!(formatter, "{operation}: {message}")
            }
            SmokeErrorKind::InvalidSample { message } => {
                write!(formatter, "invalid sample: {message}")
            }
            SmokeErrorKind::Transport { message } => write!(formatter, "transport: {message}"),
            SmokeErrorKind::Validation { message } => write!(formatter, "validation: {message}"),
            SmokeErrorKind::Wire { message } => write!(formatter, "wire: {message}"),
            SmokeErrorKind::Usage { message } => formatter.write_str(message),
        }
    }
}

impl Error for SmokeError {}

/// Machine-readable AV1 LAN smoke harness error variants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SmokeErrorKind {
    /// File, socket, or artifact I/O failed.
    Io {
        /// Operation being performed.
        operation: &'static str,
        /// Underlying I/O error text.
        message: String,
    },
    /// The checked-in sample was missing or malformed.
    InvalidSample {
        /// Validation failure text.
        message: String,
    },
    /// Existing transport sample validation rejected the payload.
    Transport {
        /// Transport error text.
        message: String,
    },
    /// Receiver validation rejected metadata or payload bytes.
    Validation {
        /// Validation failure text.
        message: String,
    },
    /// Wire encoding or decoding failed.
    Wire {
        /// Wire failure text.
        message: String,
    },
    /// CLI or address usage was invalid.
    Usage {
        /// Usage failure text.
        message: String,
    },
}
