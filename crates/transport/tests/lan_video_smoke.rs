#![doc = "Localhost integration tests for the AV1 LAN smoke harness."]

use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread;

use madobe_transport::video_smoke::{
    CHECKED_IN_AV1_SHA256, SmokeErrorKind, SmokeSample, receive_checked_av1_sample,
    send_checked_av1_sample,
};

#[test]
fn sender_and_receiver_exchange_checked_in_av1_sample_on_localhost() {
    let sample = sample_path();
    let evidence_dir = test_dir("exchange");
    let listener = must(TcpListener::bind("127.0.0.1:0"), "bind localhost listener");
    let addr = must(listener.local_addr(), "read listener address");
    let receiver_dir = evidence_dir.join("receiver");
    let sender_dir = evidence_dir.join("sender");
    let receiver_thread =
        thread::spawn(move || receive_checked_av1_sample(&listener, Some(&receiver_dir)));

    let sent = must(
        send_checked_av1_sample(addr, sample, Some(&sender_dir)),
        "send checked sample",
    );
    let received = must(
        must(receiver_thread.join(), "join receiver"),
        "receive checked sample",
    );

    assert_eq!(sent.payload_bytes, 84);
    assert_eq!(sent.metadata.payload_hash.value, CHECKED_IN_AV1_SHA256);
    assert_eq!(received.payload_bytes, 84);
    assert_eq!(received.payload_sha256, CHECKED_IN_AV1_SHA256);
    assert!(received.passed);
    assert_file(sender_dir.join("sender.log"));
    assert_file(sender_dir.join("sender-timeline.json"));
    assert_file(evidence_dir.join("receiver/receiver.log"));
    assert_file(evidence_dir.join("receiver/receiver-timeline.json"));
    assert_file(evidence_dir.join("receiver/result.json"));
}

#[test]
fn sample_loader_rejects_non_sample_payload_hash() {
    let dir = test_dir("bad-sample");
    must(fs::create_dir_all(&dir), "create bad sample dir");
    let bad_sample = dir.join("bad.ivf");
    let mut payload = vec![0_u8; 40];
    payload[0..4].copy_from_slice(b"DKIF");
    payload[6..8].copy_from_slice(&32_u16.to_le_bytes());
    payload[8..12].copy_from_slice(b"AV01");
    payload[12..14].copy_from_slice(&160_u16.to_le_bytes());
    payload[14..16].copy_from_slice(&90_u16.to_le_bytes());
    must(fs::write(&bad_sample, payload), "write bad sample");

    let Err(error) = SmokeSample::load(&bad_sample) else {
        panic!("non-sample payload should be rejected");
    };

    assert!(matches!(
        error.kind(),
        SmokeErrorKind::InvalidSample { message } if message.contains("sha256")
    ));
}

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/m2-nvenc-encode-sample/sample-av1.ivf")
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "madobe-m3-lan-video-smoke-harness-{name}-{}",
        std::process::id()
    ));
    if dir.exists() {
        must(fs::remove_dir_all(&dir), "clear stale test dir");
    }
    must(fs::create_dir_all(&dir), "create test dir");

    dir
}

fn assert_file(path: impl AsRef<Path>) {
    let path = path.as_ref();
    assert!(path.is_file(), "{} should exist", path.display());
}

fn must<T, E: std::fmt::Debug>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error:?}"),
    }
}
