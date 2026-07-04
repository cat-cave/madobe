#!/usr/bin/env bash
set -euo pipefail

error_count=0

report_error() {
  printf 'qd report check: %s: %s\n' "$1" "$2" >&2
  error_count=$((error_count + 1))
}

json_ok() {
  local file=$1

  if ! jq -e . "$file" >/dev/null; then
    report_error "$file" "invalid JSON"
    return 1
  fi
}

expect_jq() {
  local file=$1
  local expression=$2
  local message=$3

  if ! jq -e "$expression" "$file" >/dev/null; then
    report_error "$file" "$message"
  fi
}

node_id_for_report() {
  local file=$1
  local parent

  parent=${file%/*}
  printf '%s\n' "${parent##*/}"
}

validate_real_world_status() {
  local file=$1

  expect_jq \
    "$file" \
    '(.realWorldValidation.status | type == "string") and (.realWorldValidation.status == "passed" or .realWorldValidation.status == "not_required")' \
    "realWorldValidation.status must be passed or not_required"
}

validate_node_id() {
  local file=$1
  local expected_node_id

  expected_node_id=$(node_id_for_report "$file")
  if ! jq -e --arg expected "$expected_node_id" '.nodeId == $expected' "$file" >/dev/null; then
    report_error "$file" "nodeId must match containing directory"
  fi
}

validate_completion() {
  local file=$1

  json_ok "$file" || return
  validate_node_id "$file"
  validate_real_world_status "$file"

  expect_jq "$file" 'type == "object"' "top level must be an object"
  expect_jq "$file" '.nodeId | type == "string" and length > 0' "nodeId must be a non-empty string"
  expect_jq "$file" '.summary | type == "string"' "summary must be a string"
  expect_jq "$file" '.changedFiles | type == "array" and all(.[]; type == "string")' "changedFiles must be an array of strings"
  expect_jq "$file" '.commits | type == "array" and all(.[]; type == "string")' "commits must be an array of strings"
  expect_jq "$file" '.acceptanceEvidence | type == "array" and all(.[]; type == "object" and (.criterion | type == "string") and (.status | type == "string") and (.evidence | type == "string"))' "acceptanceEvidence must be an array of evidence objects"
  expect_jq "$file" '.commandsRun | type == "array" and all(.[]; type == "object" and (.command | type == "string") and (.status | type == "string") and (.evidence | type == "string"))' "commandsRun must be an array of command objects"
  expect_jq "$file" '.evidence | type == "array" and all(.[]; type == "string")' "evidence must be an array of strings"
  expect_jq "$file" '.realWorldValidation | type == "object" and (.required | type == "boolean") and (.evidence | type == "string")' "realWorldValidation must include required boolean and evidence string"
  expect_jq "$file" '.unverifiedItems | type == "array" and length == 0' "unverifiedItems must be present and empty"
  expect_jq "$file" '.dagChangesNeeded | type == "array"' "dagChangesNeeded must be an array"
  validate_completion_evidence_paths "$file"
}

validate_completion_evidence_paths() {
  local file=$1
  local path

  while IFS= read -r path; do
    if is_ignored_evidence_text "$path"; then
      continue
    fi
    if [[ ! -e $path ]]; then
      report_error "$file" "evidence path does not exist: $path"
    fi
  done < <(jq -r '.evidence[]' "$file")
}

is_ignored_evidence_text() {
  local value=$1

  [[ -z $value ]] && return 0
  [[ $value =~ ^https?:// ]] && return 0
  [[ $value =~ ^[A-Za-z][A-Za-z0-9+.-]*: ]] && return 0
  [[ $value = /* ]] && return 0
  [[ $value =~ [[:space:]] ]] && return 0
  [[ $value == *'`'* ]] && return 0
  [[ $value == *\'* ]] && return 0
  [[ $value == *\"* ]] && return 0
  [[ $value == *\;* ]] && return 0
  [[ $value == *\,* ]] && return 0
  [[ $value == *\(* ]] && return 0
  [[ $value == *\)* ]] && return 0
  [[ $value == *:* ]] && return 0

  [[ $value == ./* ]] && return 1
  [[ $value == ../* ]] && return 1
  [[ $value == */* ]] && return 1
  [[ $value == *.* ]] && return 1

  return 0
}

validate_audit() {
  local file=$1

  json_ok "$file" || return
  validate_node_id "$file"
  validate_real_world_status "$file"

  expect_jq "$file" 'type == "object"' "top level must be an object"
  expect_jq "$file" '.nodeId | type == "string" and length > 0' "nodeId must be a non-empty string"
  expect_jq "$file" '.acceptanceReviewed | type == "array" and all(.[]; type == "object" and (.criterion | type == "string") and (.status | type == "string") and (.evidence | type == "string"))' "acceptanceReviewed must be an array of review objects"
  expect_jq "$file" '.verificationEvidence | type == "object"' "verificationEvidence must be an object"
  expect_jq "$file" '.realWorldValidation | type == "object" and (.required | type == "boolean") and (.evidence | type == "string")' "realWorldValidation must include required boolean and evidence string"
  expect_jq "$file" '.findings | type == "array"' "findings must be an array"
}

main() {
  local completion_count=0
  local audit_count=0
  local file

  command -v jq >/dev/null 2>&1 || {
    printf 'qd report check: required tool missing: jq\n' >&2
    exit 127
  }

  while IFS= read -r -d '' file; do
    completion_count=$((completion_count + 1))
    validate_completion "$file"
  done < <(find reports/qd -mindepth 2 -maxdepth 2 -type f -name completion.json -print0 | sort -z)

  while IFS= read -r -d '' file; do
    audit_count=$((audit_count + 1))
    validate_audit "$file"
  done < <(find reports/qd -mindepth 2 -maxdepth 2 -type f -name audit.json -print0 | sort -z)

  if [[ $completion_count -eq 0 ]]; then
    report_error reports/qd "no completion reports found"
  fi
  if [[ $audit_count -eq 0 ]]; then
    report_error reports/qd "no audit reports found"
  fi

  if [[ $error_count -ne 0 ]]; then
    printf 'qd report check: failed with %d error(s)\n' "$error_count" >&2
    exit 1
  fi

  printf 'qd report check: validated %d completion report(s) and %d audit report(s)\n' "$completion_count" "$audit_count"
}

main "$@"
