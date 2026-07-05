#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$script_dir/result-check-common.sh"

default_fixture="crates/protocol/fixtures/m4-product-quic-smoke/result.json"
check_name="product quic result check"

main() {
  result_check_require_jq "$check_name"
  result_check_cd_repo_root

  if [[ $# -eq 0 ]]; then
    validate_file "$default_fixture" fixture
    run_self_tests
    printf 'product quic result check: validated %s and negative fixtures\n' "$default_fixture"
    return
  fi

  local file
  for file in "$@"; do
    validate_file "$file" explicit
  done
  printf 'product quic result check: validated %d file(s)\n' "$#"
}

validate_file() {
  local file=$1
  local mode=${2:-explicit}
  local errors=0

  if [[ ! -f $file ]]; then
    printf '%s: %s: file missing\n' "$check_name" "$file" >&2
    return 1
  fi
  if ! jq empty "$file" >/dev/null; then
    printf '%s: %s: invalid JSON\n' "$check_name" "$file" >&2
    return 1
  fi

  require_jq "$file" '.nodeId | type == "string" and length > 0' 'nodeId must be a non-empty string'
  require_jq "$file" '.branch | type == "string" and length > 0' 'branch must be a non-empty string'
  require_jq "$file" '.transport == "quic"' 'transport must be quic'
  require_jq "$file" '.productQuic == true' 'productQuic must be true'
  require_jq "$file" '.sender.role == "sender"' 'sender.role must be sender'
  require_jq "$file" '.receiver.role == "receiver"' 'receiver.role must be receiver'
  require_jq "$file" '.sender.platform | type == "string" and length > 0' 'sender.platform must be set'
  require_jq "$file" '.receiver.platform | type == "string" and length > 0' 'receiver.platform must be set'
  require_jq "$file" '.sender.evidenceDir | type == "string" and length > 0' 'sender.evidenceDir must be set'
  require_jq "$file" '.receiver.evidenceDir | type == "string" and length > 0' 'receiver.evidenceDir must be set'
  require_jq "$file" '.sender.evidenceDir | is_madobe_repo_relative_reference' 'sender.evidenceDir must be repo-relative and traversal-free'
  require_jq "$file" '.receiver.evidenceDir | is_madobe_repo_relative_reference' 'receiver.evidenceDir must be repo-relative and traversal-free'
  require_jq "$file" '.payload.payloadBytes | type == "number" and . > 0' 'payload.payloadBytes must be positive'
  require_jq "$file" '.payload.sha256 | type == "string" and test("^[0-9a-f]{64}$")' 'payload.sha256 must be lowercase SHA-256 hex'
  require_jq "$file" '.payload.byteCountValidated == true' 'payload byte count must be validated'
  require_jq "$file" '.payload.sha256Validated == true' 'payload SHA-256 must be validated'
  require_jq "$file" '.receiverAck.received == true' 'receiverAck.received must be true'
  require_jq "$file" '.receiverAck.payloadBytes == .payload.payloadBytes' 'receiverAck payload bytes must match payload'
  require_jq "$file" '.receiverAck.sha256 == .payload.sha256' 'receiverAck SHA-256 must match payload'
  require_jq "$file" '.certificateFingerprintSha256 == null or ((.certificateFingerprintSha256 | type) == "string" and (.certificateFingerprintSha256 | test("^[0-9a-f]{64}$")))' 'certificateFingerprintSha256 must be null or lowercase SHA-256 hex'
  require_jq "$file" '(.downstreamClaims.decoded | type) == "boolean"' 'downstreamClaims.decoded must be boolean'
  require_jq "$file" '(.downstreamClaims.rendered | type) == "boolean"' 'downstreamClaims.rendered must be boolean'
  require_jq "$file" '(.downstreamClaims.presented | type) == "boolean"' 'downstreamClaims.presented must be boolean'
  require_jq "$file" '.downstreamClaims.latencyMs == null or ((.downstreamClaims.latencyMs | type) == "number" and .downstreamClaims.latencyMs >= 0)' 'downstreamClaims.latencyMs must be null or non-negative'
  require_jq "$file" '.artifacts | type == "array" and length > 0' 'artifacts must be a non-empty array'
  require_jq "$file" 'all(.artifacts[]; (.path | type == "string" and length > 0) and (.kind | type == "string" and length > 0))' 'artifacts must include non-empty path and kind'
  require_jq "$file" 'all(.artifacts[]; .path | is_madobe_repo_relative_reference)' 'artifact paths must be repo-relative and traversal-free'
  require_jq "$file" 'all(.artifacts[]; .kind | is_product_quic_artifact_kind)' 'artifact kinds must use the product QUIC vocabulary'
  require_jq "$file" '.notes | type == "string" and length > 0' 'notes must be a non-empty string'
  require_jq "$file" '(has("framesSent") or has("framesReceived") or has("validated") or has("nonClaims")) | not' 'obsolete flat product QUIC schema fields are forbidden'
  require_jq "$file" '(.downstreamClaims.decoded != true) or any(.artifacts[]?; .kind == "decode_evidence")' 'decoded claim requires decode_evidence artifact'
  require_jq "$file" '(.downstreamClaims.rendered != true) or any(.artifacts[]?; .kind == "render_evidence")' 'rendered claim requires render_evidence artifact'
  require_jq "$file" '(.downstreamClaims.presented != true) or any(.artifacts[]?; .kind == "presentation_evidence")' 'presented claim requires presentation_evidence artifact'
  require_jq "$file" '(.downstreamClaims.latencyMs == null) or any(.artifacts[]?; .kind == "latency_evidence")' 'latency claim requires latency_evidence artifact'

  if [[ $mode == explicit ]]; then
    result_check_validate_existing_artifacts \
      "$check_name" \
      "$file" \
      "commands_log,sender_log,receiver_log,payload_validation_evidence,notes" ||
      errors=$((errors + 1))
    validate_explicit_log_content "$file" ||
      errors=$((errors + 1))
  fi

  if [[ $errors -ne 0 ]]; then
    return 1
  fi
}

require_jq() {
  local file=$1
  local filter=$2
  local message=$3

  if ! jq -e "$product_quic_jq_prelude $filter" "$file" >/dev/null; then
    printf '%s: %s: %s\n' "$check_name" "$file" "$message" >&2
    errors=$((errors + 1))
  fi
}

artifact_path_for_kind() {
  local file=$1
  local kind=$2

  jq -r --arg kind "$kind" '[.artifacts[]? | select(.kind == $kind) | .path] | first // ""' "$file"
}

require_log_token() {
  local result_file=$1
  local log_file=$2
  local token=$3
  local context=$4

  if ! grep -Fq -- "$token" "$log_file"; then
    printf '%s: %s: %s missing stable log token %s in %s\n' \
      "$check_name" "$result_file" "$context" "$token" "$log_file" >&2
    return 1
  fi
}

validate_explicit_log_content() {
  local file=$1
  local log_errors=0
  local sender_log
  local receiver_log
  local payload_bytes
  local payload_sha256
  local cert_sha256

  sender_log=$(artifact_path_for_kind "$file" sender_log)
  receiver_log=$(artifact_path_for_kind "$file" receiver_log)
  payload_bytes=$(jq -r '.payload.payloadBytes' "$file")
  payload_sha256=$(jq -r '.payload.sha256' "$file")
  cert_sha256=$(jq -r '.certificateFingerprintSha256 // ""' "$file")

  if [[ -z $sender_log ]]; then
    printf '%s: %s: explicit result must include a sender_log artifact\n' "$check_name" "$file" >&2
    log_errors=$((log_errors + 1))
  elif [[ -f $sender_log ]]; then
    require_log_token "$file" "$sender_log" "payload_bytes=$payload_bytes" sender_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$sender_log" "payload_sha256=$payload_sha256" sender_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$sender_log" "receiver_certificate_path=" sender_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$sender_log" "receiver_certificate_trusted=true" sender_log || log_errors=$((log_errors + 1))
    if [[ -z $cert_sha256 ]]; then
      printf '%s: %s: explicit sender certificate evidence requires certificateFingerprintSha256\n' "$check_name" "$file" >&2
      log_errors=$((log_errors + 1))
    else
      require_log_token "$file" "$sender_log" "receiver_certificate_sha256=$cert_sha256" sender_log || log_errors=$((log_errors + 1))
    fi
  fi

  if [[ -z $receiver_log ]]; then
    printf '%s: %s: explicit result must include a receiver_log artifact\n' "$check_name" "$file" >&2
    log_errors=$((log_errors + 1))
  elif [[ -f $receiver_log ]]; then
    require_log_token "$file" "$receiver_log" "transport=quic" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "product_quic=true" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "payload_byte_count_validated=true" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "payload_bytes=$payload_bytes" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "payload_sha256_validated=true" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "payload_sha256=$payload_sha256" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "receiver_ack=true" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "receiver_ack_payload_bytes=$payload_bytes" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "receiver_ack_sha256=$payload_sha256" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "downstream_decoded=false" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "downstream_rendered=false" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "downstream_presented=false" receiver_log || log_errors=$((log_errors + 1))
    require_log_token "$file" "$receiver_log" "downstream_latency_ms=null" receiver_log || log_errors=$((log_errors + 1))
  fi

  [[ $log_errors -eq 0 ]]
}

# shellcheck disable=SC2016,SC2154 # jq expands jq variables; helper defines common prelude.
product_quic_jq_prelude="${result_check_common_jq_prelude}"'
def product_quic_artifact_kinds:
  [
    "commands_log",
    "sender_log",
    "receiver_log",
    "payload_validation_evidence",
    "decode_evidence",
    "render_evidence",
    "presentation_evidence",
    "latency_evidence",
    "notes",
    "other"
  ];

def is_product_quic_artifact_kind:
  . as $kind
  | product_quic_artifact_kinds
  | index($kind) != null;
'

run_self_tests() {
  local tmpdir
  tmpdir=$(mktemp -d "${TMPDIR:-/tmp}/madobe-product-quic-result-check.XXXXXX")
  mkdir -p target

  local repo_tmpdir
  repo_tmpdir=$(mktemp -d "target/madobe-product-quic-result-check.XXXXXX")

  local obsolete="$tmpdir/obsolete-flat-result.json"
  local unsupported_claim="$tmpdir/unsupported-claim-result.json"
  local absolute_artifact="$tmpdir/absolute-artifact-result.json"
  local traversal_artifact="$tmpdir/traversal-artifact-result.json"
  local traversal_evidence_dir="$tmpdir/traversal-evidence-dir-result.json"
  local unknown_artifact_kind="$tmpdir/unknown-artifact-kind-result.json"
  local missing_artifact="$tmpdir/missing-artifact-result.json"
  local empty_core_artifact="$tmpdir/empty-core-artifact-result.json"
  local explicit_positive="$tmpdir/explicit-positive-result.json"
  local missing_sender_log_token="$tmpdir/missing-sender-log-token-result.json"
  local missing_receiver_log_token="$tmpdir/missing-receiver-log-token-result.json"

  local commands_log="$repo_tmpdir/commands.log"
  local sender_log="$repo_tmpdir/sender.log"
  local receiver_log="$repo_tmpdir/receiver.log"
  local bad_sender_log="$repo_tmpdir/bad-sender.log"
  local bad_receiver_log="$repo_tmpdir/bad-receiver.log"
  local payload_validation="$repo_tmpdir/payload-validation.json"
  local notes_artifact="$repo_tmpdir/notes.md"
  printf 'product QUIC commands\n' >"$commands_log"
  cat >"$sender_log" <<'SENDER_LOG'
role=sender
payload_bytes=84
payload_sha256=3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a
receiver_certificate_path=evidence/m4-product-quic-cross-device-smoke/macos-receiver/server-cert.der
receiver_certificate_sha256=9b44d90fb42f6c3ff8510ce40bbfcb1cf8712a2d18a3552955aa1b889ad2c6f3
receiver_certificate_trusted=true
SENDER_LOG
  cat >"$receiver_log" <<'RECEIVER_LOG'
role=receiver
transport=quic
product_quic=true
payload_byte_count_validated=true
payload_bytes=84
payload_sha256_validated=true
payload_sha256=3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a
receiver_ack=true
receiver_ack_payload_bytes=84
receiver_ack_sha256=3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a
downstream_decoded=false
downstream_rendered=false
downstream_presented=false
downstream_latency_ms=null
RECEIVER_LOG
  grep -Fv 'receiver_certificate_sha256=' "$sender_log" >"$bad_sender_log"
  grep -Fv 'transport=quic' "$receiver_log" >"$bad_receiver_log"
  printf '{"validated":true}\n' >"$payload_validation"
  printf 'notes\n' >"$notes_artifact"

  jq \
    --arg commands_log "$commands_log" \
    --arg sender_log "$sender_log" \
    --arg receiver_log "$receiver_log" \
    --arg payload_validation "$payload_validation" \
    --arg notes_artifact "$notes_artifact" \
    '.artifacts = [
      {"path": $commands_log, "kind": "commands_log"},
      {"path": $sender_log, "kind": "sender_log"},
      {"path": $receiver_log, "kind": "receiver_log"},
      {"path": $payload_validation, "kind": "payload_validation_evidence"},
      {"path": $notes_artifact, "kind": "notes"}
    ]' \
    "$default_fixture" >"$explicit_positive"

  cp "$explicit_positive" "$empty_core_artifact"
  : >"$sender_log"
  cp "$bad_sender_log" "$sender_log"
  jq \
    --arg sender_log "$bad_sender_log" \
    '.artifacts |= map(if .kind == "sender_log" then .path = $sender_log else . end)' \
    "$explicit_positive" >"$missing_sender_log_token"
  cp "$bad_receiver_log" "$receiver_log"
  jq \
    --arg receiver_log "$bad_receiver_log" \
    '.artifacts |= map(if .kind == "receiver_log" then .path = $receiver_log else . end)' \
    "$explicit_positive" >"$missing_receiver_log_token"
  cp "$bad_sender_log" "$sender_log"
  : >"$sender_log"

  jq \
    --arg commands_log "$commands_log" \
    --arg missing_artifact "$repo_tmpdir/missing-receiver.log" \
    '.artifacts = [
      {"path": $commands_log, "kind": "commands_log"},
      {"path": $missing_artifact, "kind": "receiver_log"}
    ]' \
    "$default_fixture" >"$missing_artifact"

  jq '. + {framesSent: 1, validated: {payloadSha256: true}}' "$default_fixture" >"$obsolete"
  jq '.downstreamClaims.decoded = true' "$default_fixture" >"$unsupported_claim"
  jq '.artifacts[0].path = "/tmp/product-quic/commands.log"' "$default_fixture" >"$absolute_artifact"
  jq '.artifacts[0].path = "evidence/../secrets/commands.log"' "$default_fixture" >"$traversal_artifact"
  jq '.receiver.evidenceDir = "evidence/../secrets"' "$default_fixture" >"$traversal_evidence_dir"
  jq '.artifacts[0].kind = "packet_capture"' "$default_fixture" >"$unknown_artifact_kind"

  local self_test_status=0
  expect_invalid "$obsolete" 'obsolete flat schema' || self_test_status=1
  expect_invalid "$unsupported_claim" 'unsupported decoded claim' || self_test_status=1
  expect_invalid "$absolute_artifact" 'absolute artifact path' || self_test_status=1
  expect_invalid "$traversal_artifact" 'traversal artifact path' || self_test_status=1
  expect_invalid "$traversal_evidence_dir" 'traversal endpoint evidence dir' || self_test_status=1
  expect_invalid "$unknown_artifact_kind" 'unknown artifact kind' || self_test_status=1
  expect_invalid "$missing_artifact" 'missing explicit artifact' explicit || self_test_status=1
  expect_invalid "$empty_core_artifact" 'empty explicit core artifact' explicit || self_test_status=1
  cp "$bad_sender_log" "$sender_log"
  expect_invalid "$missing_sender_log_token" 'missing sender log token' explicit || self_test_status=1
  cp "$bad_receiver_log" "$receiver_log"
  expect_invalid "$missing_receiver_log_token" 'missing receiver log token' explicit || self_test_status=1
  cp "$bad_sender_log" "$sender_log"
  cat >"$sender_log" <<'SENDER_LOG'
role=sender
payload_bytes=84
payload_sha256=3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a
receiver_certificate_path=evidence/m4-product-quic-cross-device-smoke/macos-receiver/server-cert.der
receiver_certificate_sha256=9b44d90fb42f6c3ff8510ce40bbfcb1cf8712a2d18a3552955aa1b889ad2c6f3
receiver_certificate_trusted=true
SENDER_LOG
  cat >"$receiver_log" <<'RECEIVER_LOG'
role=receiver
transport=quic
product_quic=true
payload_byte_count_validated=true
payload_bytes=84
payload_sha256_validated=true
payload_sha256=3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a
receiver_ack=true
receiver_ack_payload_bytes=84
receiver_ack_sha256=3d746c6c4b5f7bd72d35f4ab673f33f3e5f9a0c9f6f8b27f35fb6fbb1c3e8d2a
downstream_decoded=false
downstream_rendered=false
downstream_presented=false
downstream_latency_ms=null
RECEIVER_LOG
  expect_valid "$explicit_positive" 'positive explicit log content' explicit || self_test_status=1
  rm -rf "$tmpdir"
  rm -rf "$repo_tmpdir"
  return "$self_test_status"
}

expect_invalid() {
  local file=$1
  local context=$2
  local mode=${3:-fixture}

  if validate_file "$file" "$mode" >/dev/null 2>&1; then
    printf '%s: negative fixture unexpectedly passed: %s\n' "$check_name" "$context" >&2
    return 1
  fi
}

expect_valid() {
  local file=$1
  local context=$2
  local mode=${3:-fixture}

  if ! validate_file "$file" "$mode" >/dev/null 2>&1; then
    printf '%s: positive fixture unexpectedly failed: %s\n' "$check_name" "$context" >&2
    return 1
  fi
}

main "$@"
