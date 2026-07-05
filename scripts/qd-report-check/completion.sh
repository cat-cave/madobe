# shellcheck shell=bash
# shellcheck disable=SC1091,SC2154
# shellcheck source=scripts/qd-report-check/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

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
  validate_completion_done_node_commits "$file"
  validate_completion_commit_objects "$file"
  expect_jq "$file" '.acceptanceEvidence | type == "array" and length > 0 and all(.[]; type == "object" and (.criterion | nonblank) and (.status | nonblank) and (.evidence | nonblank))' "acceptanceEvidence must be a non-empty array of evidence objects with non-blank criterion, status, and evidence"
  expect_jq "$file" '.commandsRun | type == "array" and length > 0 and all(.[]; type == "object" and (.command | nonblank) and (.status | nonblank) and (.evidence | nonblank))' "commandsRun must be a non-empty array of command objects with non-blank command, status, and evidence"
  expect_jq "$file" '.evidence | type == "array" and length > 0 and all(.[]; nonblank)' "evidence must be a non-empty array of non-blank strings"
  expect_jq "$file" '.realWorldValidation | type == "object" and (.required | type == "boolean") and (.evidence | nonblank)' "realWorldValidation must include required boolean and non-blank evidence string"
  expect_jq "$file" '.unverifiedItems | type == "array" and length == 0' "unverifiedItems must be present and empty"
  expect_jq "$file" '.dagChangesNeeded | type == "array"' "dagChangesNeeded must be an array"
  validate_completion_changed_files "$file"
  validate_completion_evidence_paths "$file"
}

validate_completion_done_node_commits() {
  local file=$1
  local node_id

  if [[ ! -f $roadmap_export ]]; then
    return
  fi

  if ! jq -e 'type == "object" and (.nodes | type == "array")' "$roadmap_export" >/dev/null 2>&1; then
    return
  fi

  node_id=$(node_id_for_report "$file")
  if jq -e --arg node_id "$node_id" 'any(.nodes[]; .id == $node_id and .status == "done")' "$roadmap_export" >/dev/null; then
    expect_jq "$file" '.commits | type == "array" and length > 0' "commits must be non-empty when roadmap node status is done"
  fi
}

validate_completion_commit_objects() {
  local file=$1
  local sha

  if ! jq -e '.commits | type == "array" and all(.[]; type == "string" and test("^[0-9a-f]{40}$"))' "$file" >/dev/null; then
    return
  fi

  while IFS= read -r sha; do
    if ! git cat-file -e "$sha^{commit}" 2>/dev/null; then
      report_error "$file" "commits[] SHA is not present as a commit in the current git repository: $sha"
      continue
    fi
    if ! git merge-base --is-ancestor "$sha" HEAD 2>/dev/null; then
      report_error "$file" "commits[] SHA is not reachable from HEAD: $sha"
    fi
  done < <(jq -r '.commits[]' "$file")
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
