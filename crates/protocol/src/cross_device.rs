/// Artifact role attached to a cross-device video smoke result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossDeviceResultArtifactKind {
    /// Shell command transcript for the validation node.
    CommandsLog,
    /// Linux host-side smoke log.
    LinuxHostLog,
    /// macOS client-side smoke log.
    MacClientLog,
    /// Evidence that encoded frames were decoded by the client.
    DecodeEvidence,
    /// Evidence that decoded frames reached the render path.
    RenderEvidence,
    /// Evidence that rendered frames were presented onscreen.
    PresentationEvidence,
    /// Evidence that supports glass-to-glass latency measurements.
    LatencyEvidence,
    /// Human-readable validation notes.
    Notes,
    /// Any additional artifact not used as metric proof.
    Other,
}

impl CrossDeviceResultArtifactKind {
    /// Returns the stable lower-case wire token for this artifact kind.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::CommandsLog => "commands_log",
            Self::LinuxHostLog => "linux_host_log",
            Self::MacClientLog => "mac_client_log",
            Self::DecodeEvidence => "decode_evidence",
            Self::RenderEvidence => "render_evidence",
            Self::PresentationEvidence => "presentation_evidence",
            Self::LatencyEvidence => "latency_evidence",
            Self::Notes => "notes",
            Self::Other => "other",
        }
    }
}

/// Artifact referenced by a cross-device result file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossDeviceResultArtifact {
    /// Relative path from the repository root.
    pub path: String,
    /// Artifact role used by result validation.
    pub kind: CrossDeviceResultArtifactKind,
}

/// Metrics emitted by `m3-cross-device-video-smoke/result.json`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CrossDeviceVideoSmokeMetrics {
    /// Frames emitted by the Linux sender.
    pub frames_sent: u64,
    /// Frames decoded by the macOS receiver.
    pub frames_decoded: u64,
    /// Frames accepted by the macOS render path.
    pub frames_rendered: u64,
    /// Frames presented by the macOS display path.
    pub frames_presented: u64,
    /// Median glass-to-glass latency in milliseconds, if measured.
    pub median_glass_to_glass_ms: Option<u64>,
    /// P95 glass-to-glass latency in milliseconds, if measured.
    pub p95_glass_to_glass_ms: Option<u64>,
}

/// Checked result shape for `m3-cross-device-video-smoke/result.json`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossDeviceVideoSmokeResult {
    /// QD node identifier.
    pub node_id: String,
    /// Branch used by both orchestrators.
    pub branch: String,
    /// Linux worktree commit recorded by the Linux orchestrator.
    pub linux_commit: String,
    /// macOS worktree commit recorded by the macOS orchestrator.
    pub macos_commit: String,
    /// UTC start timestamp for the validation attempt.
    pub started_at: String,
    /// UTC end timestamp for the validation attempt.
    pub ended_at: String,
    /// Pass/fail state for the validation attempt.
    pub passed: bool,
    /// Cross-device metrics.
    pub metrics: CrossDeviceVideoSmokeMetrics,
    /// Artifacts referenced by the result.
    pub artifacts: Vec<CrossDeviceResultArtifact>,
    /// Free-form notes. This must not be used to imply unsupported metrics.
    pub notes: String,
}

/// Cross-device behavior claim that requires matching evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossDeviceEvidenceClaim {
    /// A nonzero decode metric was recorded.
    Decode,
    /// A nonzero render metric was recorded.
    Render,
    /// A nonzero presentation metric was recorded.
    Presentation,
    /// A non-null latency metric was recorded.
    Latency,
}

impl CrossDeviceEvidenceClaim {
    /// Returns the artifact role required to support this claim.
    #[must_use]
    pub const fn required_artifact_kind(self) -> CrossDeviceResultArtifactKind {
        match self {
            Self::Decode => CrossDeviceResultArtifactKind::DecodeEvidence,
            Self::Render => CrossDeviceResultArtifactKind::RenderEvidence,
            Self::Presentation => CrossDeviceResultArtifactKind::PresentationEvidence,
            Self::Latency => CrossDeviceResultArtifactKind::LatencyEvidence,
        }
    }
}

/// Validation error for a cross-device video smoke result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrossDeviceResultValidationError {
    /// Required text field was empty.
    EmptyRequiredField(&'static str),
    /// Result file reference was absolute or could traverse outside the repository.
    InvalidReference(&'static str),
    /// A metric claim was present without a matching artifact role.
    MissingEvidenceForClaim(CrossDeviceEvidenceClaim),
    /// Both latency metrics were present but p95 was lower than median.
    P95LatencyBelowMedian,
}

impl CrossDeviceVideoSmokeResult {
    /// Returns validation errors for unsupported or incomplete result content.
    #[must_use]
    pub fn validate(&self) -> Vec<CrossDeviceResultValidationError> {
        let mut errors = Vec::new();

        for (field_name, value) in [
            ("node_id", self.node_id.as_str()),
            ("branch", self.branch.as_str()),
            ("linux_commit", self.linux_commit.as_str()),
            ("macos_commit", self.macos_commit.as_str()),
            ("started_at", self.started_at.as_str()),
            ("ended_at", self.ended_at.as_str()),
            ("notes", self.notes.as_str()),
        ] {
            if value.is_empty() {
                errors.push(CrossDeviceResultValidationError::EmptyRequiredField(
                    field_name,
                ));
            }
        }

        for artifact in &self.artifacts {
            if !is_repo_relative_reference(artifact.path.as_str()) {
                errors.push(CrossDeviceResultValidationError::InvalidReference(
                    "artifacts[].path",
                ));
            }
        }

        self.reject_unsupported_claim(
            self.metrics.frames_decoded > 0,
            CrossDeviceEvidenceClaim::Decode,
            &mut errors,
        );
        self.reject_unsupported_claim(
            self.metrics.frames_rendered > 0,
            CrossDeviceEvidenceClaim::Render,
            &mut errors,
        );
        self.reject_unsupported_claim(
            self.metrics.frames_presented > 0,
            CrossDeviceEvidenceClaim::Presentation,
            &mut errors,
        );
        self.reject_unsupported_claim(
            self.metrics.median_glass_to_glass_ms.is_some()
                || self.metrics.p95_glass_to_glass_ms.is_some(),
            CrossDeviceEvidenceClaim::Latency,
            &mut errors,
        );

        if let (Some(median), Some(p95)) = (
            self.metrics.median_glass_to_glass_ms,
            self.metrics.p95_glass_to_glass_ms,
        ) && p95 < median
        {
            errors.push(CrossDeviceResultValidationError::P95LatencyBelowMedian);
        }

        errors
    }

    fn reject_unsupported_claim(
        &self,
        claim_present: bool,
        claim: CrossDeviceEvidenceClaim,
        errors: &mut Vec<CrossDeviceResultValidationError>,
    ) {
        if claim_present && !self.has_artifact_kind(claim.required_artifact_kind()) {
            errors.push(CrossDeviceResultValidationError::MissingEvidenceForClaim(
                claim,
            ));
        }
    }

    fn has_artifact_kind(&self, kind: CrossDeviceResultArtifactKind) -> bool {
        self.artifacts.iter().any(|artifact| artifact.kind == kind)
    }
}

fn is_repo_relative_reference(value: &str) -> bool {
    let bytes = value.as_bytes();
    let has_windows_absolute_prefix = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\');

    !value.is_empty()
        && !value.starts_with('/')
        && !has_windows_absolute_prefix
        && !value.contains('\\')
        && !value.split('/').any(|component| component == "..")
}

#[cfg(test)]
mod tests;
