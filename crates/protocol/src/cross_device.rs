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

#[cfg(test)]
mod tests {
    use super::{
        CrossDeviceEvidenceClaim, CrossDeviceResultArtifact, CrossDeviceResultArtifactKind,
        CrossDeviceResultValidationError, CrossDeviceVideoSmokeMetrics,
        CrossDeviceVideoSmokeResult,
    };

    const GOLDEN_RESULT_JSON: &str =
        include_str!("../fixtures/m3-cross-device-video-smoke/result.json");
    const FIXTURE_COMMIT: &str = "f17c0c960f92bf11715fe2759e9e33bd6c05bee3";
    const EXPECTED_RESULT_JSON: &str = concat!(
        "{\n",
        "  \"node_id\": \"m3-cross-device-video-smoke\",\n",
        "  \"branch\": \"spec/m3-cross-device-result-schema-fixture\",\n",
        "  \"linux_commit\": \"f17c0c960f92bf11715fe2759e9e33bd6c05bee3\",\n",
        "  \"macos_commit\": \"f17c0c960f92bf11715fe2759e9e33bd6c05bee3\",\n",
        "  \"started_at\": \"2026-07-03T00:00:00Z\",\n",
        "  \"ended_at\": \"2026-07-03T00:00:00Z\",\n",
        "  \"passed\": false,\n",
        "  \"metrics\": {\n",
        "    \"frames_sent\": 0,\n",
        "    \"frames_decoded\": 0,\n",
        "    \"frames_rendered\": 0,\n",
        "    \"frames_presented\": 0,\n",
        "    \"median_glass_to_glass_ms\": null,\n",
        "    \"p95_glass_to_glass_ms\": null\n",
        "  },\n",
        "  \"artifacts\": [\n",
        "    {\n",
        "      \"path\": \"evidence/m3-cross-device-video-smoke/commands.log\",\n",
        "      \"kind\": \"commands_log\"\n",
        "    },\n",
        "    {\n",
        "      \"path\": \"evidence/m3-cross-device-video-smoke/notes.md\",\n",
        "      \"kind\": \"notes\"\n",
        "    }\n",
        "  ],\n",
        "  \"notes\": \"Schema fixture only; no live cross-device decode, render, presentation, or latency behavior is claimed.\"\n",
        "}\n",
    );

    #[test]
    fn fixture_covers_required_fields_without_live_claims() {
        let result = golden_result();

        assert_eq!(result.node_id, "m3-cross-device-video-smoke");
        assert_eq!(result.branch, "spec/m3-cross-device-result-schema-fixture");
        assert_eq!(result.linux_commit, FIXTURE_COMMIT);
        assert_eq!(result.macos_commit, FIXTURE_COMMIT);
        assert_eq!(result.started_at, "2026-07-03T00:00:00Z");
        assert_eq!(result.ended_at, "2026-07-03T00:00:00Z");
        assert!(!result.passed);
        assert_eq!(result.metrics.frames_sent, 0);
        assert_eq!(result.metrics.frames_decoded, 0);
        assert_eq!(result.metrics.frames_rendered, 0);
        assert_eq!(result.metrics.frames_presented, 0);
        assert_eq!(result.metrics.median_glass_to_glass_ms, None);
        assert_eq!(result.metrics.p95_glass_to_glass_ms, None);
        assert_eq!(result.artifacts.len(), 2);
        assert_eq!(
            result.artifacts[0].kind.wire_name(),
            CrossDeviceResultArtifactKind::CommandsLog.wire_name()
        );
        assert!(result.notes.contains("Schema fixture only"));
        assert_eq!(result.validate(), []);
        assert_eq!(GOLDEN_RESULT_JSON, EXPECTED_RESULT_JSON);
    }

    #[test]
    fn fixture_round_trips_semantically() {
        assert_eq!(encode_result_fixture(&golden_result()), GOLDEN_RESULT_JSON);
    }

    #[test]
    fn rejects_decode_render_presentation_and_latency_without_evidence() {
        let result = CrossDeviceVideoSmokeResult {
            metrics: CrossDeviceVideoSmokeMetrics {
                frames_sent: 12,
                frames_decoded: 11,
                frames_rendered: 10,
                frames_presented: 9,
                median_glass_to_glass_ms: Some(41),
                p95_glass_to_glass_ms: Some(53),
            },
            ..golden_result()
        };

        assert_eq!(
            result.validate(),
            [
                CrossDeviceResultValidationError::MissingEvidenceForClaim(
                    CrossDeviceEvidenceClaim::Decode
                ),
                CrossDeviceResultValidationError::MissingEvidenceForClaim(
                    CrossDeviceEvidenceClaim::Render
                ),
                CrossDeviceResultValidationError::MissingEvidenceForClaim(
                    CrossDeviceEvidenceClaim::Presentation
                ),
                CrossDeviceResultValidationError::MissingEvidenceForClaim(
                    CrossDeviceEvidenceClaim::Latency
                ),
            ]
        );
    }

    #[test]
    fn accepts_metric_claims_with_matching_evidence() {
        let result = CrossDeviceVideoSmokeResult {
            metrics: CrossDeviceVideoSmokeMetrics {
                frames_sent: 12,
                frames_decoded: 11,
                frames_rendered: 10,
                frames_presented: 9,
                median_glass_to_glass_ms: Some(41),
                p95_glass_to_glass_ms: Some(53),
            },
            artifacts: vec![
                artifact(
                    "mac-client.log",
                    CrossDeviceResultArtifactKind::DecodeEvidence,
                ),
                artifact(
                    "render-timeline.json",
                    CrossDeviceResultArtifactKind::RenderEvidence,
                ),
                artifact(
                    "presentation-timeline.json",
                    CrossDeviceResultArtifactKind::PresentationEvidence,
                ),
                artifact(
                    "latency-summary.json",
                    CrossDeviceResultArtifactKind::LatencyEvidence,
                ),
            ],
            ..golden_result()
        };

        assert_eq!(result.validate(), []);
    }

    #[test]
    fn rejects_invalid_latency_order() {
        let result = CrossDeviceVideoSmokeResult {
            metrics: CrossDeviceVideoSmokeMetrics {
                median_glass_to_glass_ms: Some(60),
                p95_glass_to_glass_ms: Some(45),
                ..golden_result().metrics
            },
            artifacts: vec![artifact(
                "latency-summary.json",
                CrossDeviceResultArtifactKind::LatencyEvidence,
            )],
            ..golden_result()
        };

        assert_eq!(
            result.validate(),
            [CrossDeviceResultValidationError::P95LatencyBelowMedian]
        );
    }

    fn golden_result() -> CrossDeviceVideoSmokeResult {
        CrossDeviceVideoSmokeResult {
            node_id: "m3-cross-device-video-smoke".to_owned(),
            branch: "spec/m3-cross-device-result-schema-fixture".to_owned(),
            linux_commit: FIXTURE_COMMIT.to_owned(),
            macos_commit: FIXTURE_COMMIT.to_owned(),
            started_at: "2026-07-03T00:00:00Z".to_owned(),
            ended_at: "2026-07-03T00:00:00Z".to_owned(),
            passed: false,
            metrics: CrossDeviceVideoSmokeMetrics {
                frames_sent: 0,
                frames_decoded: 0,
                frames_rendered: 0,
                frames_presented: 0,
                median_glass_to_glass_ms: None,
                p95_glass_to_glass_ms: None,
            },
            artifacts: vec![
                artifact(
                    "evidence/m3-cross-device-video-smoke/commands.log",
                    CrossDeviceResultArtifactKind::CommandsLog,
                ),
                artifact(
                    "evidence/m3-cross-device-video-smoke/notes.md",
                    CrossDeviceResultArtifactKind::Notes,
                ),
            ],
            notes: "Schema fixture only; no live cross-device decode, render, presentation, or latency behavior is claimed.".to_owned(),
        }
    }

    fn artifact(path: &str, kind: CrossDeviceResultArtifactKind) -> CrossDeviceResultArtifact {
        CrossDeviceResultArtifact {
            path: path.to_owned(),
            kind,
        }
    }

    fn encode_result_fixture(result: &CrossDeviceVideoSmokeResult) -> String {
        let median = nullable_u64_to_json(result.metrics.median_glass_to_glass_ms);
        let p95 = nullable_u64_to_json(result.metrics.p95_glass_to_glass_ms);

        format!(
            concat!(
                "{{\n",
                "  \"node_id\": \"{}\",\n",
                "  \"branch\": \"{}\",\n",
                "  \"linux_commit\": \"{}\",\n",
                "  \"macos_commit\": \"{}\",\n",
                "  \"started_at\": \"{}\",\n",
                "  \"ended_at\": \"{}\",\n",
                "  \"passed\": {},\n",
                "  \"metrics\": {{\n",
                "    \"frames_sent\": {},\n",
                "    \"frames_decoded\": {},\n",
                "    \"frames_rendered\": {},\n",
                "    \"frames_presented\": {},\n",
                "    \"median_glass_to_glass_ms\": {},\n",
                "    \"p95_glass_to_glass_ms\": {}\n",
                "  }},\n",
                "  \"artifacts\": [\n",
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
            result.linux_commit,
            result.macos_commit,
            result.started_at,
            result.ended_at,
            result.passed,
            result.metrics.frames_sent,
            result.metrics.frames_decoded,
            result.metrics.frames_rendered,
            result.metrics.frames_presented,
            median,
            p95,
            result.artifacts[0].path,
            result.artifacts[0].kind.wire_name(),
            result.artifacts[1].path,
            result.artifacts[1].kind.wire_name(),
            result.notes
        )
    }

    fn nullable_u64_to_json(value: Option<u64>) -> String {
        value.map_or_else(|| "null".to_owned(), |value| value.to_string())
    }
}
