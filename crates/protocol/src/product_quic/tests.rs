use super::{
    ProductQuicDownstreamClaim, ProductQuicEndpointRole, ProductQuicPayloadValidation,
    ProductQuicReceiverAck, ProductQuicResultArtifact, ProductQuicResultArtifactKind,
    ProductQuicResultValidationError, ProductQuicSmokeEndpoint, ProductQuicSmokeResult,
    ProductQuicTransport,
};

const GOLDEN_RESULT_JSON: &str = include_str!("../../fixtures/m4-product-quic-smoke/result.json");
const PAYLOAD_SHA256: &str = "3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a";
const CERT_SHA256: &str = "9b44d90fb42f6c3ff8510ce40bbfcb1cf8712a2d18a3552955aa1b889ad2c6f3";
const EXPECTED_RESULT_JSON: &str = concat!(
    "{\n",
    "  \"nodeId\": \"m4-product-quic-cross-device-smoke\",\n",
    "  \"branch\": \"spec/m4-product-quic-result-schema\",\n",
    "  \"transport\": \"quic\",\n",
    "  \"productQuic\": true,\n",
    "  \"sender\": {\n",
    "    \"role\": \"sender\",\n",
    "    \"platform\": \"linux\",\n",
    "    \"evidenceDir\": \"evidence/m4-product-quic-cross-device-smoke/linux-sender\"\n",
    "  },\n",
    "  \"receiver\": {\n",
    "    \"role\": \"receiver\",\n",
    "    \"platform\": \"macos\",\n",
    "    \"evidenceDir\": \"evidence/m4-product-quic-cross-device-smoke/macos-receiver\"\n",
    "  },\n",
    "  \"payload\": {\n",
    "    \"payloadBytes\": 84,\n",
    "    \"sha256\": \"3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a\",\n",
    "    \"byteCountValidated\": true,\n",
    "    \"sha256Validated\": true\n",
    "  },\n",
    "  \"receiverAck\": {\n",
    "    \"received\": true,\n",
    "    \"payloadBytes\": 84,\n",
    "    \"sha256\": \"3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a\"\n",
    "  },\n",
    "  \"certificateFingerprintSha256\": \"9b44d90fb42f6c3ff8510ce40bbfcb1cf8712a2d18a3552955aa1b889ad2c6f3\",\n",
    "  \"downstreamClaims\": {\n",
    "    \"decoded\": false,\n",
    "    \"rendered\": false,\n",
    "    \"presented\": false,\n",
    "    \"latencyMs\": null\n",
    "  },\n",
    "  \"artifacts\": [\n",
    "    {\n",
    "      \"path\": \"evidence/m4-product-quic-cross-device-smoke/commands.log\",\n",
    "      \"kind\": \"commands_log\"\n",
    "    },\n",
    "    {\n",
    "      \"path\": \"evidence/m4-product-quic-cross-device-smoke/linux-sender/sender.log\",\n",
    "      \"kind\": \"sender_log\"\n",
    "    },\n",
    "    {\n",
    "      \"path\": \"evidence/m4-product-quic-cross-device-smoke/macos-receiver/receiver.log\",\n",
    "      \"kind\": \"receiver_log\"\n",
    "    },\n",
    "    {\n",
    "      \"path\": \"evidence/m4-product-quic-cross-device-smoke/macos-receiver/result.json\",\n",
    "      \"kind\": \"payload_validation_evidence\"\n",
    "    }\n",
    "  ],\n",
    "  \"notes\": \"Contract fixture only; no live product QUIC, cross-device, decode, render, presentation, or latency behavior is claimed.\"\n",
    "}\n",
);

#[test]
fn product_quic_fixture_covers_required_contract_without_live_claims() {
    let result = golden_result();

    assert_eq!(result.node_id, "m4-product-quic-cross-device-smoke");
    assert_eq!(result.branch, "spec/m4-product-quic-result-schema");
    assert_eq!(result.transport, Some(ProductQuicTransport::Quic));
    assert_eq!(ProductQuicTransport::Quic.wire_name(), "quic");
    assert_eq!(result.product_quic, Some(true));
    assert_eq!(result.sender.role, ProductQuicEndpointRole::Sender);
    assert_eq!(result.sender.role.wire_name(), "sender");
    assert_eq!(result.receiver.role, ProductQuicEndpointRole::Receiver);
    assert_eq!(result.receiver.role.wire_name(), "receiver");
    assert_eq!(result.payload.payload_bytes, 84);
    assert_eq!(result.payload.sha256, PAYLOAD_SHA256);
    assert!(result.payload.byte_count_validated);
    assert!(result.payload.sha256_validated);
    assert!(result.receiver_ack.received);
    assert_eq!(
        result.receiver_ack.payload_bytes,
        result.payload.payload_bytes
    );
    assert_eq!(result.receiver_ack.sha256, result.payload.sha256);
    assert_eq!(
        result.certificate_fingerprint_sha256.as_deref(),
        Some(CERT_SHA256)
    );
    assert!(!result.decoded);
    assert!(!result.rendered);
    assert!(!result.presented);
    assert_eq!(result.latency_ms, None);
    assert_eq!(result.validate(), []);
    assert!(result.notes.contains("Contract fixture only"));
    assert_eq!(GOLDEN_RESULT_JSON, EXPECTED_RESULT_JSON);
}

#[test]
fn product_quic_fixture_round_trips_semantically() {
    assert_eq!(encode_result_fixture(&golden_result()), GOLDEN_RESULT_JSON);
}

#[test]
fn product_quic_rejects_false_product_identity() {
    let result = ProductQuicSmokeResult {
        product_quic: Some(false),
        ..golden_result()
    };

    assert_eq!(
        result.validate(),
        [ProductQuicResultValidationError::FalseProductQuicIdentity]
    );
}

#[test]
fn product_quic_rejects_missing_product_identity() {
    let result = ProductQuicSmokeResult {
        transport: None,
        product_quic: None,
        ..golden_result()
    };

    assert_eq!(
        result.validate(),
        [
            ProductQuicResultValidationError::MissingTransport,
            ProductQuicResultValidationError::MissingProductQuicIdentity
        ]
    );
}

#[test]
fn product_quic_rejects_missing_payload_validation() {
    let result = ProductQuicSmokeResult {
        payload: ProductQuicPayloadValidation {
            byte_count_validated: false,
            sha256_validated: false,
            ..golden_result().payload
        },
        ..golden_result()
    };

    assert_eq!(
        result.validate(),
        [
            ProductQuicResultValidationError::MissingPayloadByteCountValidation,
            ProductQuicResultValidationError::MissingPayloadSha256Validation
        ]
    );
}

#[test]
fn product_quic_rejects_unsupported_downstream_claims_without_evidence() {
    let result = ProductQuicSmokeResult {
        decoded: true,
        rendered: true,
        presented: true,
        latency_ms: Some(27),
        ..golden_result()
    };

    assert_eq!(
        result.validate(),
        [
            ProductQuicResultValidationError::MissingEvidenceForDownstreamClaim(
                ProductQuicDownstreamClaim::Decode
            ),
            ProductQuicResultValidationError::MissingEvidenceForDownstreamClaim(
                ProductQuicDownstreamClaim::Render
            ),
            ProductQuicResultValidationError::MissingEvidenceForDownstreamClaim(
                ProductQuicDownstreamClaim::Presentation
            ),
            ProductQuicResultValidationError::MissingEvidenceForDownstreamClaim(
                ProductQuicDownstreamClaim::Latency
            )
        ]
    );
}

fn golden_result() -> ProductQuicSmokeResult {
    ProductQuicSmokeResult {
            node_id: "m4-product-quic-cross-device-smoke".to_owned(),
            branch: "spec/m4-product-quic-result-schema".to_owned(),
            transport: Some(ProductQuicTransport::Quic),
            product_quic: Some(true),
            sender: ProductQuicSmokeEndpoint {
                role: ProductQuicEndpointRole::Sender,
                platform: "linux".to_owned(),
                evidence_dir: "evidence/m4-product-quic-cross-device-smoke/linux-sender"
                    .to_owned(),
            },
            receiver: ProductQuicSmokeEndpoint {
                role: ProductQuicEndpointRole::Receiver,
                platform: "macos".to_owned(),
                evidence_dir: "evidence/m4-product-quic-cross-device-smoke/macos-receiver"
                    .to_owned(),
            },
            payload: ProductQuicPayloadValidation {
                payload_bytes: 84,
                sha256: PAYLOAD_SHA256.to_owned(),
                byte_count_validated: true,
                sha256_validated: true,
            },
            receiver_ack: ProductQuicReceiverAck {
                received: true,
                payload_bytes: 84,
                sha256: PAYLOAD_SHA256.to_owned(),
            },
            certificate_fingerprint_sha256: Some(CERT_SHA256.to_owned()),
            decoded: false,
            rendered: false,
            presented: false,
            latency_ms: None,
            artifacts: vec![
                artifact(
                    "evidence/m4-product-quic-cross-device-smoke/commands.log",
                    ProductQuicResultArtifactKind::CommandsLog,
                ),
                artifact(
                    "evidence/m4-product-quic-cross-device-smoke/linux-sender/sender.log",
                    ProductQuicResultArtifactKind::SenderLog,
                ),
                artifact(
                    "evidence/m4-product-quic-cross-device-smoke/macos-receiver/receiver.log",
                    ProductQuicResultArtifactKind::ReceiverLog,
                ),
                artifact(
                    "evidence/m4-product-quic-cross-device-smoke/macos-receiver/result.json",
                    ProductQuicResultArtifactKind::PayloadValidationEvidence,
                ),
            ],
            notes: "Contract fixture only; no live product QUIC, cross-device, decode, render, presentation, or latency behavior is claimed.".to_owned(),
        }
}

fn artifact(path: &str, kind: ProductQuicResultArtifactKind) -> ProductQuicResultArtifact {
    ProductQuicResultArtifact {
        path: path.to_owned(),
        kind,
    }
}

fn encode_result_fixture(result: &ProductQuicSmokeResult) -> String {
    let transport = transport_to_json(result.transport);
    let product_quic = nullable_bool_to_json(result.product_quic);
    let cert = nullable_string_to_json(result.certificate_fingerprint_sha256.as_deref());
    let latency_ms = nullable_u64_to_json(result.latency_ms);

    format!(
        concat!(
            "{{\n",
            "  \"nodeId\": \"{}\",\n",
            "  \"branch\": \"{}\",\n",
            "  \"transport\": {},\n",
            "  \"productQuic\": {},\n",
            "  \"sender\": {{\n",
            "    \"role\": \"{}\",\n",
            "    \"platform\": \"{}\",\n",
            "    \"evidenceDir\": \"{}\"\n",
            "  }},\n",
            "  \"receiver\": {{\n",
            "    \"role\": \"{}\",\n",
            "    \"platform\": \"{}\",\n",
            "    \"evidenceDir\": \"{}\"\n",
            "  }},\n",
            "  \"payload\": {{\n",
            "    \"payloadBytes\": {},\n",
            "    \"sha256\": \"{}\",\n",
            "    \"byteCountValidated\": {},\n",
            "    \"sha256Validated\": {}\n",
            "  }},\n",
            "  \"receiverAck\": {{\n",
            "    \"received\": {},\n",
            "    \"payloadBytes\": {},\n",
            "    \"sha256\": \"{}\"\n",
            "  }},\n",
            "  \"certificateFingerprintSha256\": {},\n",
            "  \"downstreamClaims\": {{\n",
            "    \"decoded\": {},\n",
            "    \"rendered\": {},\n",
            "    \"presented\": {},\n",
            "    \"latencyMs\": {}\n",
            "  }},\n",
            "  \"artifacts\": [\n",
            "    {{\n",
            "      \"path\": \"{}\",\n",
            "      \"kind\": \"{}\"\n",
            "    }},\n",
            "    {{\n",
            "      \"path\": \"{}\",\n",
            "      \"kind\": \"{}\"\n",
            "    }},\n",
            "    {{\n",
            "      \"path\": \"{}\",\n",
            "      \"kind\": \"{}\"\n",
            "    }},\n",
            "    {{\n",
            "      \"path\": \"{}\",\n",
            "      \"kind\": \"{}\"\n",
            "    }}\n",
            "  ],\n",
            "  \"notes\": \"{}\"\n",
            "}}\n",
        ),
        result.node_id,
        result.branch,
        transport,
        product_quic,
        result.sender.role.wire_name(),
        result.sender.platform,
        result.sender.evidence_dir,
        result.receiver.role.wire_name(),
        result.receiver.platform,
        result.receiver.evidence_dir,
        result.payload.payload_bytes,
        result.payload.sha256,
        result.payload.byte_count_validated,
        result.payload.sha256_validated,
        result.receiver_ack.received,
        result.receiver_ack.payload_bytes,
        result.receiver_ack.sha256,
        cert,
        result.decoded,
        result.rendered,
        result.presented,
        latency_ms,
        result.artifacts[0].path,
        result.artifacts[0].kind.wire_name(),
        result.artifacts[1].path,
        result.artifacts[1].kind.wire_name(),
        result.artifacts[2].path,
        result.artifacts[2].kind.wire_name(),
        result.artifacts[3].path,
        result.artifacts[3].kind.wire_name(),
        result.notes
    )
}

fn nullable_u64_to_json(value: Option<u64>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn nullable_bool_to_json(value: Option<bool>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn nullable_string_to_json(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| format!("\"{value}\""))
}

fn transport_to_json(value: Option<ProductQuicTransport>) -> String {
    value.map_or_else(
        || "null".to_owned(),
        |transport| nullable_string_to_json(Some(transport.wire_name())),
    )
}
