#!/usr/bin/env bash
set -euo pipefail

error_count=0
roadmap_export=roadmap/qd-export.json
jq_report_predicates='def nonblank: type == "string" and (gsub("^[[:space:]]+|[[:space:]]+$"; "") | length > 0);'

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

  if ! jq -e "$jq_report_predicates $expression" "$file" >/dev/null; then
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

validate_completion_status_values() {
  local file=$1

  expect_jq \
    "$file" \
    '(.acceptanceEvidence | type != "array" or all(.[]; type != "object" or (.status | type != "string") or (.status == "passed" or .status == "failed" or .status == "not_required")))' \
    "acceptanceEvidence.status must be passed, failed, or not_required"
  expect_jq \
    "$file" \
    '(.commandsRun | type != "array" or all(.[]; type != "object" or (.status | type != "string") or (.status == "passed" or .status == "failed" or .status == "not-run")))' \
    "commandsRun.status must be passed, failed, or not-run"
}

validate_audit_status_values() {
  local file=$1

  expect_jq \
    "$file" \
    '(.acceptanceReviewed | type != "array" or all(.[]; type != "object" or (.status | type != "string") or (.status == "passed" or .status == "failed" or .status == "not_required")))' \
    "acceptanceReviewed.status must be passed, failed, or not_required"
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
  validate_completion_status_values "$file"

  expect_jq "$file" 'type == "object"' "top level must be an object"
  expect_jq "$file" '.nodeId | nonblank' "nodeId must be a non-blank string"
  expect_jq "$file" '.summary | nonblank' "summary must be a non-blank string"
  expect_jq "$file" '.changedFiles | type == "array" and length > 0 and all(.[]; nonblank)' "changedFiles must be a non-empty array of non-blank strings"
  expect_jq "$file" '.commits | type == "array" and all(.[]; type == "string" and test("^[0-9a-f]{40}$"))' "commits must be an array of 40-character lowercase hexadecimal SHAs"
  expect_jq "$file" '.acceptanceEvidence | type == "array" and length > 0 and all(.[]; type == "object" and (.criterion | nonblank) and (.status | nonblank) and (.evidence | nonblank))' "acceptanceEvidence must be a non-empty array of evidence objects with non-blank criterion, status, and evidence"
  expect_jq "$file" '.commandsRun | type == "array" and length > 0 and all(.[]; type == "object" and (.command | nonblank) and (.status | nonblank) and (.evidence | nonblank))' "commandsRun must be a non-empty array of command objects with non-blank command, status, and evidence"
  expect_jq "$file" '.evidence | type == "array" and length > 0 and all(.[]; nonblank)' "evidence must be a non-empty array of non-blank strings"
  expect_jq "$file" '.realWorldValidation | type == "object" and (.required | type == "boolean") and (.evidence | nonblank)' "realWorldValidation must include required boolean and non-blank evidence string"
  expect_jq "$file" '.unverifiedItems | type == "array" and length == 0' "unverifiedItems must be present and empty"
  expect_jq "$file" '.dagChangesNeeded | type == "array"' "dagChangesNeeded must be an array"
  validate_completion_changed_files "$file"
  validate_completion_evidence_paths "$file"
}

validate_completion_changed_files() {
  local file=$1
  local entry
  local path

  if ! jq -e '.changedFiles | type == "array" and all(.[]; type == "string")' "$file" >/dev/null; then
    return
  fi

  while IFS= read -r entry; do
    if [[ $entry == deleted:* ]]; then
      path=${entry#deleted:}
      validate_deleted_changed_file "$file" "$entry" "$path"
    else
      validate_plain_changed_file "$file" "$entry"
    fi
  done < <(jq -r '.changedFiles[]' "$file")
}

validate_plain_changed_file() {
  local file=$1
  local path=$2

  if ! validate_repo_relative_changed_file_path "$file" "$path" "changedFiles path"; then
    return
  fi

  if [[ ! -e $path ]]; then
    report_error "$file" "changedFiles path does not exist: $path"
  fi
}

validate_deleted_changed_file() {
  local file=$1
  local entry=$2
  local path=$3

  if ! validate_repo_relative_changed_file_path "$file" "$path" "deleted changedFiles path"; then
    return
  fi

  if [[ -e $path ]]; then
    report_error "$file" "deleted changedFiles path still exists: $entry"
  fi
}

validate_repo_relative_changed_file_path() {
  local file=$1
  local path=$2
  local label=$3

  if [[ -z $path ]]; then
    report_error "$file" "$label must not be empty"
    return 1
  fi

  if [[ $path = /* ]]; then
    report_error "$file" "$label must be repo-relative: $path"
    return 1
  fi

  if path_has_traversal "$path"; then
    report_error "$file" "$label must not contain '..' traversal: $path"
    return 1
  fi

  return 0
}

path_has_traversal() {
  local path=$1

  [[ $path == .. || $path == ../* || $path == */.. || $path == */../* ]]
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
  validate_audit_status_values "$file"

  expect_jq "$file" 'type == "object"' "top level must be an object"
  expect_jq "$file" '.nodeId | nonblank' "nodeId must be a non-blank string"
  expect_jq "$file" '.acceptanceReviewed | type == "array" and length > 0 and all(.[]; type == "object" and (.criterion | nonblank) and (.status | nonblank) and (.evidence | nonblank))' "acceptanceReviewed must be a non-empty array of review objects with non-blank criterion, status, and evidence"
  expect_jq "$file" '.verificationEvidence | type == "object"' "verificationEvidence must be an object"
  expect_jq "$file" '.verificationEvidence | (.diffReviewed | type == "boolean") and (.completionReportReviewed | type == "boolean") and (.verificationEvidenceReviewed | type == "boolean")' "verificationEvidence must include boolean diffReviewed, completionReportReviewed, and verificationEvidenceReviewed"
  expect_jq "$file" '.realWorldValidation | type == "object" and (.required | type == "boolean") and (.evidence | nonblank)' "realWorldValidation must include required boolean and non-blank evidence string"
  expect_jq "$file" '.findings | type == "array"' "findings must be an array"
}

validate_roadmap_export() {
  if [[ ! -f $roadmap_export ]]; then
    report_error "$roadmap_export" "required roadmap export is missing"
    return 1
  fi

  json_ok "$roadmap_export" || return 1

  if ! jq -e 'type == "object" and (.nodes | type == "array")' "$roadmap_export" >/dev/null; then
    report_error "$roadmap_export" "must be an object with a nodes array"
    return 1
  fi

  if ! jq -e 'all(.nodes[]; (.id | type == "string" and length > 0) and (.status | type == "string" and length > 0))' "$roadmap_export" >/dev/null; then
    report_error "$roadmap_export" "each node must include non-empty string id and status fields"
    return 1
  fi

  if ! jq -e '([.nodes[].id] | length) == ([.nodes[].id] | unique | length)' "$roadmap_export" >/dev/null; then
    report_error "$roadmap_export" "node ids must be unique"
    return 1
  fi
}

node_exists_in_roadmap() {
  local node_id=$1

  jq -e --arg node_id "$node_id" 'any(.nodes[]; .id == $node_id)' "$roadmap_export" >/dev/null
}

validate_report_directory_coverage() {
  local file
  local node_id
  local previous_node_id=

  while IFS= read -r -d '' file; do
    node_id=$(node_id_for_report "$file")
    if [[ $node_id == "$previous_node_id" ]]; then
      continue
    fi
    previous_node_id=$node_id

    if ! node_exists_in_roadmap "$node_id"; then
      report_error "reports/qd/$node_id" "report directory must match a node id in $roadmap_export"
    fi
  done < <(find reports/qd -mindepth 2 -maxdepth 2 -type f \( -name completion.json -o -name audit.json \) -print0 | sort -z)
}

validate_done_node_report_coverage() {
  local node_id

  while IFS= read -r node_id; do
    if [[ ! -f reports/qd/$node_id/completion.json ]]; then
      report_error "reports/qd/$node_id" "done node is missing completion.json"
    fi

    if [[ ! -f reports/qd/$node_id/audit.json ]]; then
      report_error "reports/qd/$node_id" "done node is missing audit.json"
    fi
  done < <(jq -r '.nodes[] | select(.status == "done") | .id' "$roadmap_export" | sort)
}

main() {
  local completion_count=0
  local audit_count=0
  local file
  local roadmap_valid=0

  command -v jq >/dev/null 2>&1 || {
    printf 'qd report check: required tool missing: jq\n' >&2
    exit 127
  }

  if validate_roadmap_export; then
    roadmap_valid=1
  fi

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

  if [[ $roadmap_valid -eq 1 ]]; then
    validate_report_directory_coverage
    validate_done_node_report_coverage
  fi

  if [[ $error_count -ne 0 ]]; then
    printf 'qd report check: failed with %d error(s)\n' "$error_count" >&2
    exit 1
  fi

  printf 'qd report check: validated %d completion report(s) and %d audit report(s)\n' "$completion_count" "$audit_count"
}

main "$@"
