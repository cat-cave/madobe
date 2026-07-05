# shellcheck shell=bash

: "${error_count:=0}"
: "${roadmap_export:=${QD_REPORTS_ROADMAP_EXPORT:-roadmap/qd-export.json}}"
jq_report_predicates=${jq_report_predicates:-'def nonblank: type == "string" and (gsub("^[[:space:]]+|[[:space:]]+$"; "") | length > 0);'}

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

validate_node_id() {
  local file=$1
  local expected_node_id

  expected_node_id=$(node_id_for_report "$file")
  if ! jq -e --arg expected "$expected_node_id" '.nodeId == $expected' "$file" >/dev/null; then
    report_error "$file" "nodeId must match containing directory"
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
