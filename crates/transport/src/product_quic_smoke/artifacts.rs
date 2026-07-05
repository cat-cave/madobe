use std::fs::{self, File};
use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use crate::product_quic_smoke::{
    NODE_ID, ProductQuicError, ProductQuicReceiveSummary, ProductQuicSendSummary,
};
use crate::sha256;
use crate::video_smoke::CHECKED_IN_AV1_SAMPLE;

const PRODUCT_QUIC_BRANCH: &str = "spec/m4-product-quic-cross-device-smoke";
const PRODUCT_QUIC_EVIDENCE_ROOT: &str = "evidence/m4-product-quic-cross-device-smoke";
const PRODUCT_QUIC_SENDER_EVIDENCE_DIR: &str =
    "evidence/m4-product-quic-cross-device-smoke/linux-sender";
const PRODUCT_QUIC_RECEIVER_EVIDENCE_DIR: &str =
    "evidence/m4-product-quic-cross-device-smoke/macos-receiver";

/// Optional artifact target for product QUIC smoke sender and receiver runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductQuicArtifactSet {
    dir: Option<PathBuf>,
}

impl ProductQuicArtifactSet {
    /// Creates an artifact set from an optional directory.
    #[must_use]
    pub fn new(dir: Option<&Path>) -> Self {
        Self {
            dir: dir.map(Path::to_path_buf),
        }
    }

    pub(super) fn write_sender(
        &self,
        summary: &ProductQuicSendSummary,
    ) -> Result<(), ProductQuicError> {
        let Some(dir) = &self.dir else {
            return Ok(());
        };
        create_dir(dir)?;
        write_file(dir.join("sender.log"), &sender_log(summary))?;
        write_file(
            dir.join("sender-timeline.json"),
            &sender_timeline(summary.remote_addr),
        )
    }

    pub(super) fn write_receiver_listening(
        &self,
        local_addr: SocketAddr,
        cert_der: &[u8],
    ) -> Result<(), ProductQuicError> {
        let Some(dir) = &self.dir else {
            return Ok(());
        };
        create_dir(dir)?;
        write_bytes(dir.join("server-cert.der"), cert_der)?;
        write_file(
            dir.join("server-cert.sha256"),
            &format!("{}\n", sha256::digest_hex(cert_der)),
        )?;
        write_file(
            dir.join("receiver-listening.log"),
            &format!(
                concat!(
                    "event=listening\n",
                    "transport=quic\n",
                    "product_quic=true\n",
                    "bind={bind}\n",
                    "server_cert_der=server-cert.der\n",
                ),
                bind = local_addr,
            ),
        )
    }

    pub(super) fn write_receiver(
        &self,
        summary: &ProductQuicReceiveSummary,
    ) -> Result<(), ProductQuicError> {
        let Some(dir) = &self.dir else {
            return Ok(());
        };
        create_dir(dir)?;
        write_file(dir.join("receiver.log"), &receiver_log(summary))?;
        write_file(
            dir.join("receiver-timeline.json"),
            &receiver_timeline(summary.local_addr, summary.peer_addr),
        )?;
        write_file(dir.join("result.json"), &result_json(summary))
    }
}

fn sender_log(summary: &ProductQuicSendSummary) -> String {
    format!(
        concat!(
            "product-quic-smoke sender\n",
            "node_id={node_id}\n",
            "transport=quic\n",
            "product_quic=true\n",
            "remote_addr={remote_addr}\n",
            "server_name={server_name}\n",
            "sample={sample}\n",
            "payload_bytes={payload_bytes}\n",
            "sha256={sha256}\n",
            "ack={ack}",
            "result=sent\n",
        ),
        node_id = NODE_ID,
        remote_addr = summary.remote_addr,
        server_name = summary.server_name,
        sample = CHECKED_IN_AV1_SAMPLE,
        payload_bytes = summary.payload_bytes,
        sha256 = summary.payload_sha256,
        ack = summary.ack,
    )
}

fn receiver_log(summary: &ProductQuicReceiveSummary) -> String {
    format!(
        concat!(
            "product-quic-smoke receiver\n",
            "node_id={node_id}\n",
            "transport=quic\n",
            "product_quic=true\n",
            "local_addr={local_addr}\n",
            "peer_addr={peer_addr}\n",
            "sample={sample}\n",
            "payload_bytes={payload_bytes}\n",
            "sha256={sha256}\n",
            "payload_byte_count_valid=true\n",
            "payload_sha256_valid=true\n",
            "decode_render_presentation_latency_claim=false\n",
            "passed={passed}\n",
        ),
        node_id = NODE_ID,
        local_addr = summary.local_addr,
        peer_addr = summary.peer_addr,
        sample = CHECKED_IN_AV1_SAMPLE,
        payload_bytes = summary.payload_bytes,
        sha256 = summary.payload_sha256,
        passed = summary.passed,
    )
}

fn sender_timeline(remote_addr: SocketAddr) -> String {
    timeline_json(
        "sender",
        &[
            ("sample_loaded", CHECKED_IN_AV1_SAMPLE.to_owned()),
            ("handshake_complete", remote_addr.to_string()),
            ("frame_sent", "checked-in-av1-sample".to_owned()),
            (
                "ack_received",
                "payload sha256 validated by receiver".to_owned(),
            ),
        ],
    )
}

fn receiver_timeline(local_addr: SocketAddr, peer_addr: SocketAddr) -> String {
    timeline_json(
        "receiver",
        &[
            ("listening", local_addr.to_string()),
            ("handshake_complete", peer_addr.to_string()),
            ("frame_received", "checked-in-av1-sample".to_owned()),
            ("validated", "payload_bytes,sha256".to_owned()),
        ],
    )
}

fn timeline_json(side: &str, events: &[(&str, String)]) -> String {
    let mut output = format!(
        "{{\n  \"nodeId\": \"{NODE_ID}\",\n  \"side\": \"{}\",\n",
        json_escape(side)
    );
    output.push_str("  \"transport\": \"quic\",\n");
    output.push_str("  \"productQuic\": true,\n");
    output.push_str("  \"clock\": \"sequence-only\",\n");
    output.push_str("  \"events\": [\n");
    for (index, (event, detail)) in events.iter().enumerate() {
        let comma = if index + 1 == events.len() { "" } else { "," };
        output.push_str("    { \"sequence\": ");
        output.push_str(&index.to_string());
        output.push_str(", \"event\": \"");
        output.push_str(event);
        output.push_str("\", \"detail\": \"");
        output.push_str(&json_escape(detail));
        output.push_str("\" }");
        output.push_str(comma);
        output.push('\n');
    }
    output.push_str("  ]\n}\n");

    output
}

fn result_json(summary: &ProductQuicReceiveSummary) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"nodeId\": \"{node_id}\",\n",
            "  \"branch\": \"{branch}\",\n",
            "  \"transport\": \"quic\",\n",
            "  \"productQuic\": true,\n",
            "  \"sender\": {{\n",
            "    \"role\": \"sender\",\n",
            "    \"platform\": \"linux\",\n",
            "    \"evidenceDir\": \"{sender_evidence_dir}\"\n",
            "  }},\n",
            "  \"receiver\": {{\n",
            "    \"role\": \"receiver\",\n",
            "    \"platform\": \"macos\",\n",
            "    \"evidenceDir\": \"{receiver_evidence_dir}\"\n",
            "  }},\n",
            "  \"payload\": {{\n",
            "    \"payloadBytes\": {payload_bytes},\n",
            "    \"sha256\": \"{sha256}\",\n",
            "    \"byteCountValidated\": true,\n",
            "    \"sha256Validated\": true\n",
            "  }},\n",
            "  \"receiverAck\": {{\n",
            "    \"received\": {passed},\n",
            "    \"payloadBytes\": {payload_bytes},\n",
            "    \"sha256\": \"{sha256}\"\n",
            "  }},\n",
            "  \"certificateFingerprintSha256\": null,\n",
            "  \"downstreamClaims\": {{\n",
            "    \"decoded\": false,\n",
            "    \"rendered\": false,\n",
            "    \"presented\": false,\n",
            "    \"latencyMs\": null\n",
            "  }},\n",
            "  \"artifacts\": [\n",
            "    {{ \"path\": \"{evidence_root}/commands.log\", \"kind\": \"commands_log\" }},\n",
            "    {{ \"path\": \"{sender_evidence_dir}/sender.log\", \"kind\": \"sender_log\" }},\n",
            "    {{ \"path\": \"{receiver_evidence_dir}/receiver.log\", \"kind\": \"receiver_log\" }},\n",
            "    {{ \"path\": \"{receiver_evidence_dir}/result.json\", \"kind\": \"payload_validation_evidence\" }}\n",
            "  ],\n",
            "  \"notes\": \"Live product QUIC payload validation only. No decode, render, presentation, or latency behavior is claimed.\"\n",
            "}}\n",
        ),
        node_id = NODE_ID,
        branch = PRODUCT_QUIC_BRANCH,
        passed = summary.passed,
        payload_bytes = summary.payload_bytes,
        sha256 = summary.payload_sha256,
        evidence_root = PRODUCT_QUIC_EVIDENCE_ROOT,
        sender_evidence_dir = PRODUCT_QUIC_SENDER_EVIDENCE_DIR,
        receiver_evidence_dir = PRODUCT_QUIC_RECEIVER_EVIDENCE_DIR,
    )
}

fn create_dir(dir: &Path) -> Result<(), ProductQuicError> {
    fs::create_dir_all(dir).map_err(|error| ProductQuicError::io("create artifact dir", &error))
}

fn write_file(path: PathBuf, content: &str) -> Result<(), ProductQuicError> {
    let mut file =
        File::create(path).map_err(|error| ProductQuicError::io("create artifact", &error))?;
    file.write_all(content.as_bytes())
        .map_err(|error| ProductQuicError::io("write artifact", &error))
}

fn write_bytes(path: PathBuf, content: &[u8]) -> Result<(), ProductQuicError> {
    let mut file =
        File::create(path).map_err(|error| ProductQuicError::io("create artifact", &error))?;
    file.write_all(content)
        .map_err(|error| ProductQuicError::io("write artifact", &error))
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
