#![doc = "Loopback tests for the dependency-free transport skeleton."]

use madobe_protocol::{EncodedVideoFrameMetadata, PayloadHash, PayloadHashAlgorithm, VideoCodec};
use madobe_telemetry::Timestamp;
use madobe_transport::{
    EndpointRole, LaneId, LoopbackQuicEndpoint, SessionId, TransportErrorKind,
    TransportTelemetryDirection, TransportTelemetryLog, VIDEO_LANE_ID, VideoSample,
};

#[test]
fn loopback_sender_and_receiver_exchange_video_sample() {
    let (client, server) = LoopbackQuicEndpoint::pair(SessionId::new(9));
    let sender = client.video_lane();
    let server_lane = server.video_lane();
    let sample = sample(42, b"madobe-m3-sample-frame".to_vec());
    let mut telemetry = TransportTelemetryLog::default();

    sender.send(
        sample.clone(),
        Timestamp::from_unix_millis(1_000),
        &mut telemetry,
    );
    let delivered_sample =
        match server_lane.receive(Timestamp::from_unix_millis(1_007), &mut telemetry) {
            Ok(Some(sample)) => sample,
            Ok(None) => panic!("video lane should contain one sample"),
            Err(error) => panic!("video lane receive should not error: {error}"),
        };

    assert_eq!(client.role(), EndpointRole::Client);
    assert_eq!(server.role(), EndpointRole::Server);
    assert_eq!(client.session_id(), SessionId::new(9));
    assert_eq!(delivered_sample, sample);
    assert_eq!(
        server_lane.receive(Timestamp::from_unix_millis(1_008), &mut telemetry),
        Ok(None)
    );
}

#[test]
fn video_lane_records_send_and_receive_timestamps() {
    let (client, server) = LoopbackQuicEndpoint::pair(SessionId::new(10));
    let mut telemetry = TransportTelemetryLog::default();

    client.video_lane().send(
        sample(7, b"frame-seven".to_vec()),
        Timestamp::from_unix_millis(2_000),
        &mut telemetry,
    );
    let received = match server
        .video_lane()
        .receive(Timestamp::from_unix_millis(2_011), &mut telemetry)
    {
        Ok(received) => received,
        Err(error) => panic!("receive should not error: {error}"),
    };

    assert!(received.is_some());
    assert_eq!(
        telemetry.events(),
        &[
            event(10, 7, TransportTelemetryDirection::Send, 2_000),
            event(10, 7, TransportTelemetryDirection::Receive, 2_011),
        ]
    );
}

#[test]
fn video_sample_rejects_mismatched_payload_size_with_typed_error() {
    let mut metadata = metadata(1, 99);
    metadata.payload_bytes = 99;

    let Err(error) = VideoSample::new(metadata, b"short".to_vec()) else {
        panic!("metadata size mismatch should be a typed transport error");
    };

    assert_eq!(
        error.kind(),
        &TransportErrorKind::FramePayloadSizeMismatch {
            expected: 99,
            actual: 5,
        }
    );
}

fn sample(frame_id: u64, payload: Vec<u8>) -> VideoSample {
    match VideoSample::new(metadata(frame_id, payload.len()), payload) {
        Ok(sample) => sample,
        Err(error) => panic!("test samples should have matching metadata: {error}"),
    }
}

fn metadata(frame_id: u64, payload_len: usize) -> EncodedVideoFrameMetadata {
    let payload_bytes = match u32::try_from(payload_len) {
        Ok(payload_bytes) => payload_bytes,
        Err(error) => panic!("test payload length should fit in protocol metadata: {error}"),
    };

    EncodedVideoFrameMetadata {
        frame_id,
        codec: VideoCodec::Av1,
        width: 1280,
        height: 720,
        capture_timestamp_ns: 1_720_000_000_000_000_000 + frame_id,
        encode_timestamp_ns: 1_720_000_000_002_000_000 + frame_id,
        keyframe: frame_id == 1,
        payload_bytes,
        payload_hash: PayloadHash {
            algorithm: PayloadHashAlgorithm::Sha256,
            value: "not-validated-by-transport-skeleton".to_owned(),
        },
    }
}

const fn event(
    session_id: u64,
    frame_id: u64,
    direction: TransportTelemetryDirection,
    unix_millis: u64,
) -> madobe_transport::TransportTelemetryEvent {
    madobe_transport::TransportTelemetryEvent {
        session_id: SessionId::new(session_id),
        lane_id: VIDEO_LANE_ID,
        frame_id,
        direction,
        timestamp: Timestamp::from_unix_millis(unix_millis),
    }
}

#[test]
fn video_lane_identifier_is_stable() {
    assert_eq!(VIDEO_LANE_ID, LaneId::new(0));
}
