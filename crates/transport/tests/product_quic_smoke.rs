#![doc = "Localhost integration tests for the product QUIC smoke harness."]

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use madobe_transport::product_quic_smoke::{
    ProductQuicReceiveOptions, ProductQuicSendOptions, receive_checked_av1_sample,
    send_checked_av1_sample,
};
use madobe_transport::video_smoke::CHECKED_IN_AV1_SHA256;

#[test]
fn sender_and_receiver_exchange_checked_in_av1_sample_over_product_quic() {
    let evidence_dir = test_dir("exchange");
    let receiver_dir = evidence_dir.join("receiver");
    let sender_dir = evidence_dir.join("sender");
    let receiver_options = ProductQuicReceiveOptions {
        bind: "127.0.0.1:47141".to_owned(),
        cert_subject_alt_names: vec!["localhost".to_owned()],
        artifact_dir: Some(receiver_dir.clone()),
    };

    let receiver_thread = thread::spawn(move || receive_checked_av1_sample(&receiver_options));
    wait_for_file(receiver_dir.join("server-cert.der"));

    let sender_options = ProductQuicSendOptions {
        addr: "127.0.0.1:47141".to_owned(),
        server_name: "localhost".to_owned(),
        server_cert_der: receiver_dir.join("server-cert.der"),
        sample_path: sample_path(),
        artifact_dir: Some(sender_dir.clone()),
    };
    let sent_result = send_checked_av1_sample(&sender_options);
    let received_result = must(receiver_thread.join(), "join receiver");
    let sent = must(sent_result, "send checked sample");
    let received = must(received_result, "receive checked sample");

    assert_eq!(sent.payload_bytes, 84);
    assert_eq!(sent.payload_sha256, CHECKED_IN_AV1_SHA256);
    assert_eq!(sent.ack, format!("ok sha256={CHECKED_IN_AV1_SHA256}\n"));
    assert_eq!(received.payload_bytes, 84);
    assert_eq!(received.payload_sha256, CHECKED_IN_AV1_SHA256);
    assert!(received.passed);
    assert_file(sender_dir.join("sender.log"));
    assert_file(sender_dir.join("sender-timeline.json"));
    assert_file(receiver_dir.join("receiver-listening.log"));
    assert_file(receiver_dir.join("receiver.log"));
    assert_file(receiver_dir.join("receiver-timeline.json"));
    assert_file(receiver_dir.join("result.json"));
}

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/m2-nvenc-encode-sample/sample-av1.ivf")
}

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "madobe-m4-product-quic-cross-device-smoke-{name}-{}",
        std::process::id()
    ));
    if dir.exists() {
        must(fs::remove_dir_all(&dir), "clear stale test dir");
    }
    must(fs::create_dir_all(&dir), "create test dir");

    dir
}

fn wait_for_file(path: impl AsRef<Path>) {
    let path = path.as_ref();
    for _attempt in 0..200 {
        if path.is_file() {
            return;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("{} should exist", path.display());
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
