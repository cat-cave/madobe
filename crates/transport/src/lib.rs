#![doc = "Transport skeletons and smoke harnesses for madobe."]
#![doc = ""]
#![doc = "The core QUIC-shaped session and video lane remain in-memory only."]
#![doc = "The `video_smoke` module adds a TCP validation harness for evidence"]
#![doc = "collection, but it is not product QUIC and makes no capture, decode,"]
#![doc = "render, presentation, or latency claim. The `product_quic_smoke`"]
#![doc = "module sends the same checked-in sample over the product QUIC stack,"]
#![doc = "but still makes no decode, render, presentation, or latency claim."]
#![forbid(unsafe_code)]

pub mod product_quic_smoke;
pub mod video_smoke;

pub mod sha256;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use madobe_protocol::EncodedVideoFrameMetadata;
use madobe_telemetry::Timestamp;

/// Stable identifier for a loopback transport session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionId(u64);

impl SessionId {
    /// Creates a session identifier.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw identifier value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Stable identifier for a transport lane within a session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LaneId(u64);

impl LaneId {
    /// Creates a lane identifier.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw identifier value.
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// The first reserved video lane identifier.
pub const VIDEO_LANE_ID: LaneId = LaneId::new(0);

/// Endpoint role in a loopback session pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointRole {
    /// Client side of the loopback session.
    Client,
    /// Server side of the loopback session.
    Server,
}

/// Transport lane kind carried by an envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaneKind {
    /// Encoded video sample lane.
    Video,
}

/// Direction for a transport telemetry timestamp.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportTelemetryDirection {
    /// Timestamp recorded when a sample is enqueued for sending.
    Send,
    /// Timestamp recorded when a sample is received from the lane.
    Receive,
}

/// One transport timestamp event for evidence and tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportTelemetryEvent {
    /// Session where the event occurred.
    pub session_id: SessionId,
    /// Lane where the event occurred.
    pub lane_id: LaneId,
    /// Video frame identifier carried by the sample.
    pub frame_id: u64,
    /// Send or receive direction.
    pub direction: TransportTelemetryDirection,
    /// Deterministic event timestamp.
    pub timestamp: Timestamp,
}

/// In-memory telemetry sink used by the skeleton and loopback tests.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TransportTelemetryLog {
    events: Vec<TransportTelemetryEvent>,
}

impl TransportTelemetryLog {
    /// Records a transport telemetry event.
    pub fn record(&mut self, event: TransportTelemetryEvent) {
        self.events.push(event);
    }

    /// Returns recorded events in insertion order.
    #[must_use]
    pub fn events(&self) -> &[TransportTelemetryEvent] {
        &self.events
    }
}

/// Encoded video sample carried by the video lane.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VideoSample {
    /// Protocol metadata for the encoded frame.
    pub metadata: EncodedVideoFrameMetadata,
    /// Opaque encoded payload bytes.
    pub payload: Vec<u8>,
}

impl VideoSample {
    /// Creates a video sample and checks that metadata matches payload length.
    ///
    /// This does not verify the cryptographic payload hash; that belongs in a
    /// later protocol or security node with explicit dependency policy work.
    ///
    /// # Errors
    ///
    /// Returns [`TransportErrorKind::FramePayloadSizeMismatch`] when the
    /// metadata byte count and actual payload length differ, or
    /// [`TransportErrorKind::PayloadTooLarge`] when the payload cannot fit in
    /// the protocol `u32` byte count.
    pub fn new(
        metadata: EncodedVideoFrameMetadata,
        payload: Vec<u8>,
    ) -> Result<Self, TransportError> {
        let actual = u32::try_from(payload.len()).map_err(|_| TransportError {
            kind: TransportErrorKind::PayloadTooLarge {
                actual: payload.len(),
            },
        })?;

        if metadata.payload_bytes != actual {
            return Err(TransportError {
                kind: TransportErrorKind::FramePayloadSizeMismatch {
                    expected: metadata.payload_bytes,
                    actual,
                },
            });
        }

        Ok(Self { metadata, payload })
    }
}

/// Typed transport error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportError {
    kind: TransportErrorKind,
}

impl TransportError {
    /// Returns the typed error kind.
    #[must_use]
    pub const fn kind(&self) -> &TransportErrorKind {
        &self.kind
    }
}

impl Display for TransportError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            TransportErrorKind::PayloadTooLarge { actual } => {
                write!(formatter, "payload has {actual} bytes and exceeds u32")
            }
            TransportErrorKind::FramePayloadSizeMismatch { expected, actual } => write!(
                formatter,
                "video sample metadata declares {expected} bytes but payload has {actual} bytes"
            ),
            TransportErrorKind::UnexpectedLane {
                expected,
                actual,
                lane_id,
            } => write!(
                formatter,
                "expected {expected:?} lane but received {actual:?} on lane {}",
                lane_id.value()
            ),
        }
    }
}

impl Error for TransportError {}

/// Machine-readable transport error variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportErrorKind {
    /// Payload length cannot fit in protocol metadata.
    PayloadTooLarge {
        /// Actual payload byte length.
        actual: usize,
    },
    /// Video metadata and payload byte length differ.
    FramePayloadSizeMismatch {
        /// Metadata-declared byte count.
        expected: u32,
        /// Actual payload byte count.
        actual: u32,
    },
    /// A receiver observed a lane kind it does not handle.
    UnexpectedLane {
        /// Lane kind expected by the receiver.
        expected: LaneKind,
        /// Lane kind carried by the envelope.
        actual: LaneKind,
        /// Lane identifier carried by the envelope.
        lane_id: LaneId,
    },
}

/// One side of an in-memory QUIC-shaped loopback session.
#[derive(Clone, Debug)]
pub struct LoopbackQuicEndpoint {
    session_id: SessionId,
    role: EndpointRole,
    inbound: SharedLaneQueue,
    outbound: SharedLaneQueue,
}

impl LoopbackQuicEndpoint {
    /// Creates connected client and server endpoints for one loopback session.
    #[must_use]
    pub fn pair(session_id: SessionId) -> (Self, Self) {
        let client_to_server = SharedLaneQueue::default();
        let server_to_client = SharedLaneQueue::default();

        let client = Self {
            session_id,
            role: EndpointRole::Client,
            inbound: server_to_client.clone(),
            outbound: client_to_server.clone(),
        };
        let server = Self {
            session_id,
            role: EndpointRole::Server,
            inbound: client_to_server,
            outbound: server_to_client,
        };

        (client, server)
    }

    /// Returns this endpoint's session identifier.
    #[must_use]
    pub const fn session_id(&self) -> SessionId {
        self.session_id
    }

    /// Returns this endpoint's role.
    #[must_use]
    pub const fn role(&self) -> EndpointRole {
        self.role
    }

    /// Opens the reserved video lane on this endpoint.
    #[must_use]
    pub fn video_lane(&self) -> VideoLane {
        VideoLane {
            session_id: self.session_id,
            lane_id: VIDEO_LANE_ID,
            inbound: self.inbound.clone(),
            outbound: self.outbound.clone(),
        }
    }
}

/// In-memory video lane for framed sample payloads.
#[derive(Clone, Debug)]
pub struct VideoLane {
    session_id: SessionId,
    lane_id: LaneId,
    inbound: SharedLaneQueue,
    outbound: SharedLaneQueue,
}

impl VideoLane {
    /// Sends a video sample through the loopback lane and records telemetry.
    pub fn send(
        &self,
        sample: VideoSample,
        send_timestamp: Timestamp,
        telemetry: &mut TransportTelemetryLog,
    ) {
        telemetry.record(TransportTelemetryEvent {
            session_id: self.session_id,
            lane_id: self.lane_id,
            frame_id: sample.metadata.frame_id,
            direction: TransportTelemetryDirection::Send,
            timestamp: send_timestamp,
        });

        self.outbound.push(TransportEnvelope {
            lane_id: self.lane_id,
            lane_kind: LaneKind::Video,
            sample,
        });
    }

    /// Receives the next video sample and records telemetry when a sample exists.
    ///
    /// # Errors
    ///
    /// Returns [`TransportErrorKind::UnexpectedLane`] if the next envelope is
    /// not a video-lane envelope.
    pub fn receive(
        &self,
        receive_timestamp: Timestamp,
        telemetry: &mut TransportTelemetryLog,
    ) -> Result<Option<VideoSample>, TransportError> {
        let Some(envelope) = self.inbound.pop_front() else {
            return Ok(None);
        };

        if envelope.lane_kind != LaneKind::Video {
            return Err(TransportError {
                kind: TransportErrorKind::UnexpectedLane {
                    expected: LaneKind::Video,
                    actual: envelope.lane_kind,
                    lane_id: envelope.lane_id,
                },
            });
        }

        telemetry.record(TransportTelemetryEvent {
            session_id: self.session_id,
            lane_id: envelope.lane_id,
            frame_id: envelope.sample.metadata.frame_id,
            direction: TransportTelemetryDirection::Receive,
            timestamp: receive_timestamp,
        });

        Ok(Some(envelope.sample))
    }
}

#[derive(Clone, Debug, Default)]
struct SharedLaneQueue(Rc<RefCell<VecDeque<TransportEnvelope>>>);

impl SharedLaneQueue {
    fn push(&self, envelope: TransportEnvelope) {
        self.0.borrow_mut().push_back(envelope);
    }

    fn pop_front(&self) -> Option<TransportEnvelope> {
        self.0.borrow_mut().pop_front()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TransportEnvelope {
    lane_id: LaneId,
    lane_kind: LaneKind,
    sample: VideoSample,
}
