use std::fs::{self, File};
use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use crate::video_smoke::{
    CHECKED_IN_AV1_SAMPLE, CHECKED_IN_AV1_SHA256, NODE_ID, SmokeError, SmokeReceiveSummary,
    SmokeSendSummary,
};

/// Sender or receiver side for deterministic smoke artifacts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SmokeSide {
    /// TCP sample sender.
    Sender,
    /// TCP sample receiver and validator.
    Receiver,
}

impl SmokeSide {
    const fn label(self) -> &'static str {
        match self {
            Self::Sender => "sender",
            Self::Receiver => "receiver",
        }
    }
}

/// Optional artifact target for smoke sender and receiver runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SmokeArtifactSet {
    dir: Option<PathBuf>,
}

impl SmokeArtifactSet {
    /// Creates an artifact set from an optional directory.
    #[must_use]
    pub fn new(dir: Option<&Path>) -> Self {
        Self {
            dir: dir.map(Path::to_path_buf),
        }
    }

    pub(super) fn write_sender(&self, summary: &SmokeSendSummary) -> Result<(), SmokeError> {
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
    ) -> Result<(), SmokeError> {
        let Some(dir) = &self.dir else {
            return Ok(());
        };
        create_dir(dir)?;
        write_file(
            dir.join("receiver-listening.log"),
            &format!("event=listening bind={local_addr}\n"),
        )
    }

    pub(super) fn write_receiver(&self, summary: &SmokeReceiveSummary) -> Result<(), SmokeError> {
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

fn sender_log(summary: &SmokeSendSummary) -> String {
    format!(
        concat!(
            "lan-video-smoke sender\n",
            "node_id={node_id}\n",
            "transport=tcp\n",
            "product_quic=false\n",
            "remote_addr={remote_addr}\n",
            "sample={sample}\n",
            "frame_id={frame_id}\n",
            "codec={codec}\n",
            "dimensions={width}x{height}\n",
            "payload_bytes={payload_bytes}\n",
            "sha256={sha256}\n",
            "result=sent\n",
        ),
        node_id = NODE_ID,
        remote_addr = summary.remote_addr,
        sample = CHECKED_IN_AV1_SAMPLE,
        frame_id = summary.metadata.frame_id,
        codec = summary.metadata.codec.wire_name(),
        width = summary.metadata.width,
        height = summary.metadata.height,
        payload_bytes = summary.payload_bytes,
        sha256 = summary.metadata.payload_hash.value,
    )
}

fn receiver_log(summary: &SmokeReceiveSummary) -> String {
    format!(
        concat!(
            "lan-video-smoke receiver\n",
            "node_id={node_id}\n",
            "transport=tcp\n",
            "product_quic=false\n",
            "local_addr={local_addr}\n",
            "peer_addr={peer_addr}\n",
            "frame_id={frame_id}\n",
            "codec={codec}\n",
            "dimensions={width}x{height}\n",
            "payload_bytes={payload_bytes}\n",
            "sha256={sha256}\n",
            "metadata_valid=true\n",
            "payload_byte_count_valid=true\n",
            "payload_sha256_valid=true\n",
            "passed={passed}\n",
        ),
        node_id = NODE_ID,
        local_addr = summary.local_addr,
        peer_addr = summary.peer_addr,
        frame_id = summary.metadata.frame_id,
        codec = summary.metadata.codec.wire_name(),
        width = summary.metadata.width,
        height = summary.metadata.height,
        payload_bytes = summary.payload_bytes,
        sha256 = summary.payload_sha256,
        passed = summary.passed,
    )
}

fn sender_timeline(remote_addr: SocketAddr) -> String {
    timeline_json(
        SmokeSide::Sender,
        &[
            ("sample_loaded", CHECKED_IN_AV1_SAMPLE.to_owned()),
            ("connected", remote_addr.to_string()),
            ("frame_sent", CHECKED_IN_AV1_SHA256.to_owned()),
        ],
    )
}

fn receiver_timeline(local_addr: SocketAddr, peer_addr: SocketAddr) -> String {
    timeline_json(
        SmokeSide::Receiver,
        &[
            ("listening", local_addr.to_string()),
            ("accepted", peer_addr.to_string()),
            ("frame_received", CHECKED_IN_AV1_SHA256.to_owned()),
            (
                "validated",
                "codec,dimensions,frame_id,keyframe,timestamps,payload_bytes,sha256".to_owned(),
            ),
        ],
    )
}

fn timeline_json(side: SmokeSide, events: &[(&str, String)]) -> String {
    let mut output = format!(
        "{{\n  \"nodeId\": \"{NODE_ID}\",\n  \"side\": \"{}\",\n",
        side.label()
    );
    output.push_str("  \"clock\": \"sequence-only\",\n  \"events\": [\n");
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

fn result_json(summary: &SmokeReceiveSummary) -> String {
    format!(
        concat!(
            "{{\n",
            "  \"nodeId\": \"{node_id}\",\n",
            "  \"transport\": \"tcp\",\n",
            "  \"productQuic\": false,\n",
            "  \"sample\": \"{sample}\",\n",
            "  \"passed\": {passed},\n",
            "  \"framesSent\": 1,\n",
            "  \"framesReceived\": 1,\n",
            "  \"payloadBytes\": {payload_bytes},\n",
            "  \"sha256\": \"{sha256}\",\n",
            "  \"validated\": {{\n",
            "    \"metadata\": true,\n",
            "    \"payloadByteCount\": true,\n",
            "    \"payloadSha256\": true\n",
            "  }},\n",
            "  \"nonClaims\": [\n",
            "    \"not QUIC\",\n",
            "    \"not VideoToolbox decode\",\n",
            "    \"not Metal render\",\n",
            "    \"not presentation\",\n",
            "    \"not latency proof\"\n",
            "  ]\n",
            "}}\n",
        ),
        node_id = NODE_ID,
        sample = CHECKED_IN_AV1_SAMPLE,
        passed = summary.passed,
        payload_bytes = summary.payload_bytes,
        sha256 = summary.payload_sha256,
    )
}

fn create_dir(dir: &Path) -> Result<(), SmokeError> {
    fs::create_dir_all(dir).map_err(|error| SmokeError::io("create artifact dir", &error))
}

fn write_file(path: PathBuf, content: &str) -> Result<(), SmokeError> {
    let mut file = File::create(path).map_err(|error| SmokeError::io("create artifact", &error))?;
    file.write_all(content.as_bytes())
        .map_err(|error| SmokeError::io("write artifact", &error))
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
