#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$script_dir/result-check-common.sh"

default_fixture="crates/protocol/fixtures/m3-cross-device-video-smoke/result.json"
check_name="cross-device result check"

main() {
  result_check_require_jq "$check_name"
  result_check_cd_repo_root

  if [[ $# -eq 0 ]]; then
    validate_file "$default_fixture" fixture
    run_self_tests
    printf 'cross-device result check: validated %s and negative fixtures\n' "$default_fixture"
    return
  fi

  local file
  for file in "$@"; do
    validate_file "$file" explicit
  done
  printf 'cross-device result check: validated %d file(s)\n' "$#"
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

  require_jq "$file" '.node_id | type == "string" and length > 0' 'node_id must be a non-empty string'
  require_jq "$file" '.branch | type == "string" and length > 0' 'branch must be a non-empty string'
  require_jq "$file" '.linux_commit | type == "string" and length > 0' 'linux_commit must be a non-empty string'
  require_jq "$file" '.macos_commit | type == "string" and length > 0' 'macos_commit must be a non-empty string'
  require_jq "$file" '.started_at | type == "string" and length > 0' 'started_at must be a non-empty string'
  require_jq "$file" '.ended_at | type == "string" and length > 0' 'ended_at must be a non-empty string'
  require_jq "$file" '.passed | type == "boolean"' 'passed must be boolean'
  require_jq "$file" '.metrics | type == "object"' 'metrics must be an object'
  require_jq "$file" '.metrics.frames_sent | is_madobe_nonnegative_integer' 'metrics.frames_sent must be a non-negative integer'
  require_jq "$file" '.metrics.frames_decoded | is_madobe_nonnegative_integer' 'metrics.frames_decoded must be a non-negative integer'
  require_jq "$file" '.metrics.frames_rendered | is_madobe_nonnegative_integer' 'metrics.frames_rendered must be a non-negative integer'
  require_jq "$file" '.metrics.frames_presented | is_madobe_nonnegative_integer' 'metrics.frames_presented must be a non-negative integer'
  require_jq "$file" '.metrics.median_glass_to_glass_ms | is_madobe_nullable_nonnegative_integer' 'metrics.median_glass_to_glass_ms must be null or a non-negative integer'
  require_jq "$file" '.metrics.p95_glass_to_glass_ms | is_madobe_nullable_nonnegative_integer' 'metrics.p95_glass_to_glass_ms must be null or a non-negative integer'
  require_jq "$file" '.artifacts | type == "array" and length > 0' 'artifacts must be a non-empty array'
  require_jq "$file" 'all(.artifacts[]; (.path | type == "string" and length > 0) and (.kind | type == "string" and length > 0))' 'artifacts must include non-empty path and kind'
  require_jq "$file" 'all(.artifacts[]; .path | is_madobe_repo_relative_reference)' 'artifact paths must be repo-relative, forward-slash, and traversal-free'
  require_jq "$file" 'all(.artifacts[]; .kind | is_cross_device_artifact_kind)' 'artifact kinds must use the generic cross-device vocabulary'
  require_jq "$file" '.notes | type == "string" and length > 0' 'notes must be a non-empty string'
  require_jq "$file" '(.metrics.frames_decoded == 0) or any(.artifacts[]?; .kind == "decode_evidence")' 'nonzero frames_decoded requires decode_evidence artifact'
  require_jq "$file" '(.metrics.frames_rendered == 0) or any(.artifacts[]?; .kind == "render_evidence")' 'nonzero frames_rendered requires render_evidence artifact'
  require_jq "$file" '(.metrics.frames_presented == 0) or any(.artifacts[]?; .kind == "presentation_evidence")' 'nonzero frames_presented requires presentation_evidence artifact'
  require_jq "$file" '((.metrics.median_glass_to_glass_ms == null) and (.metrics.p95_glass_to_glass_ms == null)) or any(.artifacts[]?; .kind == "latency_evidence")' 'non-null latency metrics require latency_evidence artifact'
  require_jq "$file" '((.metrics.median_glass_to_glass_ms == null) or (.metrics.p95_glass_to_glass_ms == null) or (.metrics.p95_glass_to_glass_ms >= .metrics.median_glass_to_glass_ms))' 'p95 latency must be greater than or equal to median latency'

  if [[ $mode == explicit ]]; then
    result_check_validate_existing_artifacts \
      "$check_name" \
      "$file" \
      "commands_log,linux_host_log,mac_client_log,notes" ||
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

  if ! jq -e "$cross_device_jq_prelude $filter" "$file" >/dev/null; then
    printf '%s: %s: %s\n' "$check_name" "$file" "$message" >&2
    errors=$((errors + 1))
  fi
}

log_has_fixed_token() {
  local log_file=$1
  local token=$2

  grep -Fq -- "$token" "$log_file"
}

log_has_key_value_token() {
  local log_file=$1
  local key=$2

  grep -Eq "(^|[[:space:]])${key}=[^[:space:]]+" "$log_file"
}

log_has_pass_token() {
  local log_file=$1

  log_has_fixed_token "$log_file" "passed=true" ||
    log_has_fixed_token "$log_file" "status=passed"
}

linux_host_log_has_required_tokens() {
  local log_file=$1

  log_has_fixed_token "$log_file" "transport=tcp" &&
    log_has_fixed_token "$log_file" "product_quic=false" &&
    log_has_key_value_token "$log_file" "payload_bytes" &&
    log_has_key_value_token "$log_file" "sha256" &&
    log_has_fixed_token "$log_file" "result=sent"
}

mac_client_log_has_required_tokens() {
  local log_file=$1

  log_has_key_value_token "$log_file" "payload_bytes" &&
    log_has_key_value_token "$log_file" "sha256" &&
    log_has_fixed_token "$log_file" "payload_byte_count_valid=true" &&
    log_has_fixed_token "$log_file" "payload_sha256_valid=true" &&
    log_has_pass_token "$log_file"
}

require_matching_log_for_kind() {
  local file=$1
  local kind=$2
  local context=$3
  local matcher=$4
  local path

  while IFS= read -r path; do
    if [[ -f $path ]] && "$matcher" "$path"; then
      return 0
    fi
  done < <(jq -r --arg kind "$kind" '.artifacts[]? | select(.kind == $kind) | .path' "$file")

  printf '%s: %s: explicit result must include a %s artifact with stable %s evidence tokens\n' \
    "$check_name" "$file" "$kind" "$context" >&2
  return 1
}

validate_explicit_log_content() {
  local file=$1
  local log_errors=0
  local frames_sent
  local passed

  frames_sent=$(jq -r '.metrics.frames_sent' "$file")
  passed=$(jq -r '.passed' "$file")

  if ((frames_sent > 0)); then
    require_matching_log_for_kind \
      "$file" \
      linux_host_log \
      'Linux send' \
      linux_host_log_has_required_tokens ||
      log_errors=$((log_errors + 1))
  fi

  if [[ $passed == true ]]; then
    require_matching_log_for_kind \
      "$file" \
      mac_client_log \
      'Mac receive validation' \
      mac_client_log_has_required_tokens ||
      log_errors=$((log_errors + 1))
  fi

  [[ $log_errors -eq 0 ]]
}

# shellcheck disable=SC2016,SC2154 # jq expands jq variables; helper defines common prelude.
cross_device_jq_prelude="${result_check_common_jq_prelude}"'
def cross_device_artifact_kinds:
  [
    "commands_log",
    "linux_host_log",
    "mac_client_log",
    "decode_evidence",
    "render_evidence",
    "presentation_evidence",
    "latency_evidence",
    "notes",
    "other"
  ];

def is_cross_device_artifact_kind:
  . as $kind
  | cross_device_artifact_kinds
  | index($kind) != null;
'

run_self_tests() {
  local tmpdir
  tmpdir=$(mktemp -d "${TMPDIR:-/tmp}/madobe-cross-device-result-check.XXXXXX")
  mkdir -p target

  local repo_tmpdir
  repo_tmpdir=$(mktemp -d "target/madobe-cross-device-result-check.XXXXXX")

  local decoded_claim="$tmpdir/decoded-claim-result.json"
  local rendered_claim="$tmpdir/rendered-claim-result.json"
  local presented_claim="$tmpdir/presented-claim-result.json"
  local latency_claim="$tmpdir/latency-claim-result.json"
  local p95_below_median="$tmpdir/p95-below-median-result.json"
  local absolute_artifact="$tmpdir/absolute-artifact-result.json"
  local traversal_artifact="$tmpdir/traversal-artifact-result.json"
  local backslash_artifact="$tmpdir/backslash-artifact-result.json"
  local windows_drive_artifact="$tmpdir/windows-drive-artifact-result.json"
  local blank_artifact="$tmpdir/blank-artifact-result.json"
  local unknown_artifact_kind="$tmpdir/unknown-artifact-kind-result.json"
  local explicit_positive="$tmpdir/explicit-positive-result.json"
  local missing_linux_log_token="$tmpdir/missing-linux-log-token-result.json"
  local missing_mac_log_token="$tmpdir/missing-mac-log-token-result.json"
  local zero_frames_fixture="$tmpdir/zero-frames-result.json"

  local commands_log="$repo_tmpdir/commands.log"
  local linux_log="$repo_tmpdir/linux-host.log"
  local mac_log="$repo_tmpdir/mac-client.log"
  local bad_linux_log="$repo_tmpdir/bad-linux-host.log"
  local bad_mac_log="$repo_tmpdir/bad-mac-client.log"
  local notes_artifact="$repo_tmpdir/notes.md"
  printf 'cross-device commands\n' >"$commands_log"
  cat >"$linux_log" <<'LINUX_LOG'
lan-video-smoke sender
transport=tcp
product_quic=false
payload_bytes=84
sha256=51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875
result=sent
LINUX_LOG
  cat >"$mac_log" <<'MAC_LOG'
lan-video-smoke receiver
payload_bytes=84
sha256=51945e4cd903e28019fbbfbe74572b5d836f6ef1184cb782b142aba1d5201875
payload_byte_count_valid=true
payload_sha256_valid=true
passed=true
MAC_LOG
  grep -Fv 'transport=tcp' "$linux_log" >"$bad_linux_log"
  grep -Fv 'payload_sha256_valid=true' "$mac_log" >"$bad_mac_log"
  printf 'notes\n' >"$notes_artifact"

  jq \
    --arg commands_log "$commands_log" \
    --arg linux_log "$linux_log" \
    --arg mac_log "$mac_log" \
    --arg notes_artifact "$notes_artifact" \
    '.metrics.frames_sent = 1
      | .passed = true
      | .artifacts = [
        {"path": $commands_log, "kind": "commands_log"},
        {"path": $linux_log, "kind": "linux_host_log"},
        {"path": $mac_log, "kind": "mac_client_log"},
        {"path": $notes_artifact, "kind": "notes"}
      ]' \
    "$default_fixture" >"$explicit_positive"

  jq \
    --arg bad_linux_log "$bad_linux_log" \
    '.artifacts |= map(if .kind == "linux_host_log" then .path = $bad_linux_log else . end)' \
    "$explicit_positive" >"$missing_linux_log_token"
  jq \
    --arg bad_mac_log "$bad_mac_log" \
    '.artifacts |= map(if .kind == "mac_client_log" then .path = $bad_mac_log else . end)' \
    "$explicit_positive" >"$missing_mac_log_token"
  jq \
    --arg commands_log "$commands_log" \
    --arg notes_artifact "$notes_artifact" \
    '.metrics.frames_sent = 0
      | .passed = false
      | .artifacts = [
        {"path": $commands_log, "kind": "commands_log"},
        {"path": $notes_artifact, "kind": "notes"}
      ]' \
    "$default_fixture" >"$zero_frames_fixture"

  jq '.metrics.frames_decoded = 1' "$default_fixture" >"$decoded_claim"
  jq '.metrics.frames_rendered = 1' "$default_fixture" >"$rendered_claim"
  jq '.metrics.frames_presented = 1' "$default_fixture" >"$presented_claim"
  jq '.metrics.median_glass_to_glass_ms = 41' "$default_fixture" >"$latency_claim"
  jq '.metrics.median_glass_to_glass_ms = 60 | .metrics.p95_glass_to_glass_ms = 45 | .artifacts += [{"path": "evidence/m3-cross-device-video-smoke/latency-summary.json", "kind": "latency_evidence"}]' "$default_fixture" >"$p95_below_median"
  jq '.artifacts[0].path = "/tmp/cross-device/commands.log"' "$default_fixture" >"$absolute_artifact"
  jq '.artifacts[0].path = "evidence/../secrets/commands.log"' "$default_fixture" >"$traversal_artifact"
  jq '.artifacts[0].path = "evidence\\m3-cross-device-video-smoke\\commands.log"' "$default_fixture" >"$backslash_artifact"
  jq '.artifacts[0].path = "C:\\evidence\\commands.log"' "$default_fixture" >"$windows_drive_artifact"
  jq '.artifacts[0].path = ""' "$default_fixture" >"$blank_artifact"
  jq '.artifacts[0].kind = "packet_capture"' "$default_fixture" >"$unknown_artifact_kind"

  local self_test_status=0
  expect_invalid "$decoded_claim" 'unsupported decoded frame claim' || self_test_status=1
  expect_invalid "$rendered_claim" 'unsupported rendered frame claim' || self_test_status=1
  expect_invalid "$presented_claim" 'unsupported presented frame claim' || self_test_status=1
  expect_invalid "$latency_claim" 'unsupported latency claim' || self_test_status=1
  expect_invalid "$p95_below_median" 'p95 latency below median' || self_test_status=1
  expect_invalid "$absolute_artifact" 'absolute artifact path' || self_test_status=1
  expect_invalid "$traversal_artifact" 'traversal artifact path' || self_test_status=1
  expect_invalid "$backslash_artifact" 'backslash artifact path' || self_test_status=1
  expect_invalid "$windows_drive_artifact" 'Windows drive artifact path' || self_test_status=1
  expect_invalid "$blank_artifact" 'blank artifact path' || self_test_status=1
  expect_invalid "$unknown_artifact_kind" 'unknown artifact kind' || self_test_status=1
  expect_invalid "$missing_linux_log_token" 'missing Linux host log token' explicit || self_test_status=1
  expect_invalid "$missing_mac_log_token" 'missing Mac client log token' explicit || self_test_status=1
  expect_valid "$explicit_positive" 'positive explicit log content' explicit || self_test_status=1
  expect_valid "$zero_frames_fixture" 'schema-only explicit zero-frame result' explicit || self_test_status=1
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
