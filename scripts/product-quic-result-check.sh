#!/usr/bin/env bash
set -euo pipefail

default_fixture="crates/protocol/fixtures/m4-product-quic-smoke/result.json"

main() {
  command -v jq >/dev/null 2>&1 || {
    printf 'product quic result check: required tool missing: jq\n' >&2
    exit 127
  }

  if [[ $# -eq 0 ]]; then
    validate_file "$default_fixture"
    run_self_tests
    printf 'product quic result check: validated %s and negative fixtures\n' "$default_fixture"
    return
  fi

  local file
  for file in "$@"; do
    validate_file "$file"
  done
  printf 'product quic result check: validated %d file(s)\n' "$#"
}

validate_file() {
  local file=$1
  local errors=0

  if [[ ! -f $file ]]; then
    printf 'product quic result check: %s: file missing\n' "$file" >&2
    return 1
  fi
  if ! jq empty "$file" >/dev/null; then
    printf 'product quic result check: %s: invalid JSON\n' "$file" >&2
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
  require_jq "$file" '.notes | type == "string" and length > 0' 'notes must be a non-empty string'
  require_jq "$file" '(has("framesSent") or has("framesReceived") or has("validated") or has("nonClaims")) | not' 'obsolete flat product QUIC schema fields are forbidden'
  require_jq "$file" '(.downstreamClaims.decoded != true) or any(.artifacts[]?; .kind == "decode_evidence")' 'decoded claim requires decode_evidence artifact'
  require_jq "$file" '(.downstreamClaims.rendered != true) or any(.artifacts[]?; .kind == "render_evidence")' 'rendered claim requires render_evidence artifact'
  require_jq "$file" '(.downstreamClaims.presented != true) or any(.artifacts[]?; .kind == "presentation_evidence")' 'presented claim requires presentation_evidence artifact'
  require_jq "$file" '(.downstreamClaims.latencyMs == null) or any(.artifacts[]?; .kind == "latency_evidence")' 'latency claim requires latency_evidence artifact'

  if [[ $errors -ne 0 ]]; then
    return 1
  fi
}

require_jq() {
  local file=$1
  local filter=$2
  local message=$3

  if ! jq -e "$filter" "$file" >/dev/null; then
    printf 'product quic result check: %s: %s\n' "$file" "$message" >&2
    errors=$((errors + 1))
  fi
}

run_self_tests() {
  local tmpdir
  tmpdir=$(mktemp -d "${TMPDIR:-/tmp}/madobe-product-quic-result-check.XXXXXX")

  local obsolete="$tmpdir/obsolete-flat-result.json"
  local unsupported_claim="$tmpdir/unsupported-claim-result.json"

  jq '. + {framesSent: 1, validated: {payloadSha256: true}}' "$default_fixture" >"$obsolete"
  jq '.downstreamClaims.decoded = true' "$default_fixture" >"$unsupported_claim"

  local self_test_status=0
  expect_invalid "$obsolete" 'obsolete flat schema' || self_test_status=1
  expect_invalid "$unsupported_claim" 'unsupported decoded claim' || self_test_status=1
  rm -rf "$tmpdir"
  return "$self_test_status"
}

expect_invalid() {
  local file=$1
  local context=$2

  if validate_file "$file" >/dev/null 2>&1; then
    printf 'product quic result check: negative fixture unexpectedly passed: %s\n' "$context" >&2
    return 1
  fi
}

main "$@"
