/// Transport used by a product QUIC smoke result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductQuicTransport {
    /// Product smoke ran over QUIC.
    Quic,
}

impl ProductQuicTransport {
    /// Returns the stable lower-case wire token for this transport.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Quic => "quic",
        }
    }
}

/// Endpoint role in a product QUIC smoke result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductQuicEndpointRole {
    /// Sender endpoint that connects to the receiver.
    Sender,
    /// Receiver endpoint that listens and acknowledges payload validation.
    Receiver,
}

impl ProductQuicEndpointRole {
    /// Returns the stable lower-case wire token for this endpoint role.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Sender => "sender",
            Self::Receiver => "receiver",
        }
    }
}

/// Endpoint identity recorded by the product QUIC smoke result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductQuicSmokeEndpoint {
    /// Endpoint role.
    pub role: ProductQuicEndpointRole,
    /// Platform label used in evidence paths or logs.
    pub platform: String,
    /// Evidence directory for this endpoint.
    pub evidence_dir: String,
}

/// Payload validation fields recorded by the product QUIC smoke result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductQuicPayloadValidation {
    /// Byte count of the smoke payload.
    pub payload_bytes: u64,
    /// SHA-256 digest of the smoke payload bytes.
    pub sha256: String,
    /// True when the receiver validated the payload byte count.
    pub byte_count_validated: bool,
    /// True when the receiver validated the payload SHA-256 digest.
    pub sha256_validated: bool,
}

/// Receiver acknowledgement recorded by the product QUIC smoke result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductQuicReceiverAck {
    /// True when the receiver acknowledged the validated payload.
    pub received: bool,
    /// Payload byte count echoed by the receiver ack.
    pub payload_bytes: u64,
    /// Payload SHA-256 echoed by the receiver ack.
    pub sha256: String,
}

/// Artifact role attached to a product QUIC smoke result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductQuicResultArtifactKind {
    /// Shell command transcript for the validation node.
    CommandsLog,
    /// Linux sender log.
    SenderLog,
    /// macOS receiver log.
    ReceiverLog,
    /// Receiver ack or payload validation evidence.
    PayloadValidationEvidence,
    /// Evidence that encoded frames were decoded by the client.
    DecodeEvidence,
    /// Evidence that decoded frames reached the render path.
    RenderEvidence,
    /// Evidence that rendered frames were presented onscreen.
    PresentationEvidence,
    /// Evidence that supports latency measurements.
    LatencyEvidence,
    /// Human-readable validation notes.
    Notes,
    /// Any additional artifact not used as downstream proof.
    Other,
}

impl ProductQuicResultArtifactKind {
    /// Returns the stable lower-case wire token for this artifact kind.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::CommandsLog => "commands_log",
            Self::SenderLog => "sender_log",
            Self::ReceiverLog => "receiver_log",
            Self::PayloadValidationEvidence => "payload_validation_evidence",
            Self::DecodeEvidence => "decode_evidence",
            Self::RenderEvidence => "render_evidence",
            Self::PresentationEvidence => "presentation_evidence",
            Self::LatencyEvidence => "latency_evidence",
            Self::Notes => "notes",
            Self::Other => "other",
        }
    }
}

/// Artifact referenced by a product QUIC smoke result file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductQuicResultArtifact {
    /// Relative path from the repository root.
    pub path: String,
    /// Artifact role used by result validation.
    pub kind: ProductQuicResultArtifactKind,
}

/// Product QUIC downstream behavior claim that requires matching evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductQuicDownstreamClaim {
    /// Decode behavior was claimed.
    Decode,
    /// Render behavior was claimed.
    Render,
    /// Presentation behavior was claimed.
    Presentation,
    /// Latency behavior was claimed.
    Latency,
}

impl ProductQuicDownstreamClaim {
    /// Returns the artifact role required to support this downstream claim.
    #[must_use]
    pub const fn required_artifact_kind(self) -> ProductQuicResultArtifactKind {
        match self {
            Self::Decode => ProductQuicResultArtifactKind::DecodeEvidence,
            Self::Render => ProductQuicResultArtifactKind::RenderEvidence,
            Self::Presentation => ProductQuicResultArtifactKind::PresentationEvidence,
            Self::Latency => ProductQuicResultArtifactKind::LatencyEvidence,
        }
    }
}

/// Checked product QUIC smoke result shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductQuicSmokeResult {
    /// QD node identifier.
    pub node_id: String,
    /// Branch used for the product QUIC smoke.
    pub branch: String,
    /// Transport identity. Must be QUIC for this contract.
    pub transport: Option<ProductQuicTransport>,
    /// Product QUIC identity flag. Must be true for this contract.
    pub product_quic: Option<bool>,
    /// Sender endpoint identity.
    pub sender: ProductQuicSmokeEndpoint,
    /// Receiver endpoint identity.
    pub receiver: ProductQuicSmokeEndpoint,
    /// Payload validation performed by the receiver.
    pub payload: ProductQuicPayloadValidation,
    /// Receiver ack for the validated payload.
    pub receiver_ack: ProductQuicReceiverAck,
    /// Optional receiver certificate SHA-256 fingerprint.
    pub certificate_fingerprint_sha256: Option<String>,
    /// True only when downstream decode evidence is attached.
    pub decoded: bool,
    /// True only when downstream render evidence is attached.
    pub rendered: bool,
    /// True only when downstream presentation evidence is attached.
    pub presented: bool,
    /// Latency measurement, if separately evidenced.
    pub latency_ms: Option<u64>,
    /// Artifacts referenced by the result.
    pub artifacts: Vec<ProductQuicResultArtifact>,
    /// Free-form notes. This must not be used to imply unsupported behavior.
    pub notes: String,
}

/// Validation error for a product QUIC smoke result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProductQuicResultValidationError {
    /// Required text field was empty.
    EmptyRequiredField(&'static str),
    /// Transport identity was missing.
    MissingTransport,
    /// Transport identity was not product QUIC over QUIC.
    UnsupportedTransport,
    /// Product QUIC identity flag was missing.
    MissingProductQuicIdentity,
    /// Product QUIC identity flag was false.
    FalseProductQuicIdentity,
    /// Sender or receiver role did not match the required contract.
    InvalidEndpointRole(&'static str),
    /// Payload byte count was empty.
    EmptyPayload,
    /// Payload SHA-256 digest was missing or not lowercase hex.
    InvalidPayloadSha256,
    /// Receiver did not validate the payload byte count.
    MissingPayloadByteCountValidation,
    /// Receiver did not validate the payload SHA-256 digest.
    MissingPayloadSha256Validation,
    /// Receiver ack was missing.
    MissingReceiverAck,
    /// Receiver ack did not match the validated payload.
    ReceiverAckMismatch,
    /// Optional certificate fingerprint was present but not lowercase SHA-256 hex.
    InvalidCertificateFingerprint,
    /// A downstream claim was present without a matching artifact role.
    MissingEvidenceForDownstreamClaim(ProductQuicDownstreamClaim),
}

impl ProductQuicSmokeResult {
    /// Returns validation errors for unsupported or incomplete product QUIC result content.
    #[must_use]
    pub fn validate(&self) -> Vec<ProductQuicResultValidationError> {
        let mut errors = Vec::new();

        for (field_name, value) in [
            ("node_id", self.node_id.as_str()),
            ("branch", self.branch.as_str()),
            ("sender.platform", self.sender.platform.as_str()),
            ("sender.evidence_dir", self.sender.evidence_dir.as_str()),
            ("receiver.platform", self.receiver.platform.as_str()),
            ("receiver.evidence_dir", self.receiver.evidence_dir.as_str()),
            ("notes", self.notes.as_str()),
        ] {
            if value.is_empty() {
                errors.push(ProductQuicResultValidationError::EmptyRequiredField(
                    field_name,
                ));
            }
        }

        match self.transport {
            Some(ProductQuicTransport::Quic) => {}
            None => errors.push(ProductQuicResultValidationError::MissingTransport),
        }

        match self.product_quic {
            Some(true) => {}
            Some(false) => errors.push(ProductQuicResultValidationError::FalseProductQuicIdentity),
            None => errors.push(ProductQuicResultValidationError::MissingProductQuicIdentity),
        }

        if self.sender.role != ProductQuicEndpointRole::Sender {
            errors.push(ProductQuicResultValidationError::InvalidEndpointRole(
                "sender",
            ));
        }
        if self.receiver.role != ProductQuicEndpointRole::Receiver {
            errors.push(ProductQuicResultValidationError::InvalidEndpointRole(
                "receiver",
            ));
        }

        if self.payload.payload_bytes == 0 {
            errors.push(ProductQuicResultValidationError::EmptyPayload);
        }
        if !is_sha256_hex(self.payload.sha256.as_str()) {
            errors.push(ProductQuicResultValidationError::InvalidPayloadSha256);
        }
        if !self.payload.byte_count_validated {
            errors.push(ProductQuicResultValidationError::MissingPayloadByteCountValidation);
        }
        if !self.payload.sha256_validated {
            errors.push(ProductQuicResultValidationError::MissingPayloadSha256Validation);
        }

        if !self.receiver_ack.received {
            errors.push(ProductQuicResultValidationError::MissingReceiverAck);
        }
        if self.receiver_ack.payload_bytes != self.payload.payload_bytes
            || self.receiver_ack.sha256 != self.payload.sha256
        {
            errors.push(ProductQuicResultValidationError::ReceiverAckMismatch);
        }

        if let Some(fingerprint) = &self.certificate_fingerprint_sha256
            && !is_sha256_hex(fingerprint)
        {
            errors.push(ProductQuicResultValidationError::InvalidCertificateFingerprint);
        }

        self.reject_unsupported_downstream_claim(
            self.decoded,
            ProductQuicDownstreamClaim::Decode,
            &mut errors,
        );
        self.reject_unsupported_downstream_claim(
            self.rendered,
            ProductQuicDownstreamClaim::Render,
            &mut errors,
        );
        self.reject_unsupported_downstream_claim(
            self.presented,
            ProductQuicDownstreamClaim::Presentation,
            &mut errors,
        );
        self.reject_unsupported_downstream_claim(
            self.latency_ms.is_some(),
            ProductQuicDownstreamClaim::Latency,
            &mut errors,
        );

        errors
    }

    fn reject_unsupported_downstream_claim(
        &self,
        claim_present: bool,
        claim: ProductQuicDownstreamClaim,
        errors: &mut Vec<ProductQuicResultValidationError>,
    ) {
        if claim_present && !self.has_artifact_kind(claim.required_artifact_kind()) {
            errors.push(ProductQuicResultValidationError::MissingEvidenceForDownstreamClaim(claim));
        }
    }

    fn has_artifact_kind(&self, kind: ProductQuicResultArtifactKind) -> bool {
        self.artifacts.iter().any(|artifact| artifact.kind == kind)
    }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests;
