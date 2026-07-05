#!/usr/bin/env bash
set -euo pipefail

default_fixture="crates/protocol/fixtures/m3-cross-device-video-smoke/result.json"

main() {
  command -v jq >/dev/null 2>&1 || {
    printf 'cross-device result check: required tool missing: jq\n' >&2
    exit 127
  }

  local repo_root
  repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
  cd "$repo_root"

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
    printf 'cross-device result check: %s: file missing\n' "$file" >&2
    return 1
  fi
  if ! jq empty "$file" >/dev/null; then
    printf 'cross-device result check: %s: invalid JSON\n' "$file" >&2
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
  require_jq "$file" '.metrics.frames_sent | is_cross_device_nonnegative_integer' 'metrics.frames_sent must be a non-negative integer'
  require_jq "$file" '.metrics.frames_decoded | is_cross_device_nonnegative_integer' 'metrics.frames_decoded must be a non-negative integer'
  require_jq "$file" '.metrics.frames_rendered | is_cross_device_nonnegative_integer' 'metrics.frames_rendered must be a non-negative integer'
  require_jq "$file" '.metrics.frames_presented | is_cross_device_nonnegative_integer' 'metrics.frames_presented must be a non-negative integer'
  require_jq "$file" '.metrics.median_glass_to_glass_ms | is_cross_device_nullable_nonnegative_integer' 'metrics.median_glass_to_glass_ms must be null or a non-negative integer'
  require_jq "$file" '.metrics.p95_glass_to_glass_ms | is_cross_device_nullable_nonnegative_integer' 'metrics.p95_glass_to_glass_ms must be null or a non-negative integer'
  require_jq "$file" '.artifacts | type == "array" and length > 0' 'artifacts must be a non-empty array'
  require_jq "$file" 'all(.artifacts[]; (.path | type == "string" and length > 0) and (.kind | type == "string" and length > 0))' 'artifacts must include non-empty path and kind'
  require_jq "$file" 'all(.artifacts[]; .path | is_cross_device_repo_relative_reference)' 'artifact paths must be repo-relative, forward-slash, and traversal-free'
  require_jq "$file" 'all(.artifacts[]; .kind | is_cross_device_artifact_kind)' 'artifact kinds must use the generic cross-device vocabulary'
  require_jq "$file" '.notes | type == "string" and length > 0' 'notes must be a non-empty string'
  require_jq "$file" '(.metrics.frames_decoded == 0) or any(.artifacts[]?; .kind == "decode_evidence")' 'nonzero frames_decoded requires decode_evidence artifact'
  require_jq "$file" '(.metrics.frames_rendered == 0) or any(.artifacts[]?; .kind == "render_evidence")' 'nonzero frames_rendered requires render_evidence artifact'
  require_jq "$file" '(.metrics.frames_presented == 0) or any(.artifacts[]?; .kind == "presentation_evidence")' 'nonzero frames_presented requires presentation_evidence artifact'
  require_jq "$file" '((.metrics.median_glass_to_glass_ms == null) and (.metrics.p95_glass_to_glass_ms == null)) or any(.artifacts[]?; .kind == "latency_evidence")' 'non-null latency metrics require latency_evidence artifact'
  require_jq "$file" '((.metrics.median_glass_to_glass_ms == null) or (.metrics.p95_glass_to_glass_ms == null) or (.metrics.p95_glass_to_glass_ms >= .metrics.median_glass_to_glass_ms))' 'p95 latency must be greater than or equal to median latency'

  if [[ $mode == explicit ]]; then
    validate_explicit_artifacts "$file" || errors=$((errors + 1))
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
    printf 'cross-device result check: %s: %s\n' "$file" "$message" >&2
    errors=$((errors + 1))
  fi
}

validate_explicit_artifacts() {
  local file=$1
  local artifact_errors=0
  local path
  local kind

  while IFS=$'\t' read -r path kind; do
    if [[ ! -e $path ]]; then
      printf 'cross-device result check: %s: artifact path does not exist: %s\n' "$file" "$path" >&2
      artifact_errors=$((artifact_errors + 1))
      continue
    fi

    case "$kind" in
    commands_log | linux_host_log | mac_client_log | notes)
      if [[ ! -s $path ]]; then
        printf 'cross-device result check: %s: core evidence artifact is empty: %s\n' "$file" "$path" >&2
        artifact_errors=$((artifact_errors + 1))
      fi
      ;;
    esac
  done < <(jq -r '.artifacts[]? | [.path, .kind] | @tsv' "$file")

  [[ $artifact_errors -eq 0 ]]
}

# shellcheck disable=SC2016 # jq variables are expanded by jq, not the shell.
cross_device_jq_prelude='
def is_cross_device_nonnegative_integer:
  type == "number" and . >= 0 and floor == .;

def is_cross_device_nullable_nonnegative_integer:
  . == null or is_cross_device_nonnegative_integer;

def is_cross_device_repo_relative_reference:
  type == "string"
  and length > 0
  and (startswith("/") | not)
  and (test("^[A-Za-z]:[\\\\/]") | not)
  and (contains("\\") | not)
  and ((split("/") | index("..")) == null);

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
  rm -rf "$tmpdir"
  return "$self_test_status"
}

expect_invalid() {
  local file=$1
  local context=$2

  if validate_file "$file" fixture >/dev/null 2>&1; then
    printf 'cross-device result check: negative fixture unexpectedly passed: %s\n' "$context" >&2
    return 1
  fi
}

main "$@"
