#!/usr/bin/env bash
set -euo pipefail

error_count=0

report_error() {
  printf 'pin hygiene check: %s: %s\n' "$1" "$2" >&2
  error_count=$((error_count + 1))
}

trim() {
  local value=$1

  value=${value#"${value%%[![:space:]]*}"}
  value=${value%"${value##*[![:space:]]}"}
  printf '%s\n' "$value"
}

strip_quotes() {
  local value=$1
  local first
  local last

  if [[ ${#value} -lt 2 ]]; then
    printf '%s\n' "$value"
    return
  fi

  first=${value:0:1}
  last=${value: -1}
  if [[ $first == "$last" && ($first == \" || $first == \') ]]; then
    printf '%s\n' "${value:1:${#value}-2}"
    return
  fi

  printf '%s\n' "$value"
}

check_mise_tools() {
  local file=.mise.toml
  local in_tools=0
  local line
  local line_no=0
  local section
  local value

  [[ -f $file ]] || return 0

  while IFS= read -r line || [[ -n $line ]]; do
    line_no=$((line_no + 1))
    line=${line%%#*}
    line=$(trim "$line")

    [[ -n $line ]] || continue

    if [[ $line =~ ^\[([^]]+)\]$ ]]; then
      section=${BASH_REMATCH[1]}
      in_tools=0
      [[ $section == tools ]] && in_tools=1
      continue
    fi

    [[ $in_tools -eq 1 ]] || continue
    [[ $line == *=* ]] || continue

    value=${line#*=}
    value=$(trim "$value")
    value=$(strip_quotes "$value")

    if [[ $value == "latest" || $value == "stable" ]]; then
      report_error "$file:$line_no" "pin native tool selector instead of using $value"
    fi
  done <"$file"
}

is_allowed_action_ref() {
  local ref=$1
  local sha_ref_regex='^[0-9A-Fa-f]{40}$'

  [[ $ref =~ $sha_ref_regex ]]
}

check_action_ref() {
  local location=$1
  local uses_value=$2
  local ref

  if [[ $uses_value != *@* ]]; then
    report_error "$location" "pin external action ref for $uses_value"
    return
  fi

  ref=${uses_value##*@}
  if [[ -z $ref ]]; then
    report_error "$location" "pin external action ref for $uses_value"
    return
  fi

  if ! is_allowed_action_ref "$ref"; then
    report_error "$location" "pin external action $uses_value to a full 40-character commit SHA"
  fi
}

check_workflow_file() {
  local file=$1
  local line
  local line_no=0
  local uses_value

  while IFS= read -r line || [[ -n $line ]]; do
    line_no=$((line_no + 1))
    line=${line%%#*}
    line=$(trim "$line")

    [[ $line =~ ^-?[[:space:]]*uses:[[:space:]]*(.+)$ ]] || continue

    uses_value=${BASH_REMATCH[1]}
    uses_value=$(trim "$uses_value")
    uses_value=$(strip_quotes "$uses_value")

    [[ $uses_value == ./* ]] && continue
    [[ $uses_value == docker://* ]] && continue

    check_action_ref "$file:$line_no" "$uses_value"
  done <"$file"
}

check_workflows() {
  local file

  [[ -d .github/workflows ]] || return 0

  while IFS= read -r -d '' file; do
    check_workflow_file "$file"
  done < <(find .github/workflows -maxdepth 1 -type f \( -name '*.yml' -o -name '*.yaml' \) -print0 | sort -z)
}

main() {
  check_mise_tools
  check_workflows

  if [[ $error_count -ne 0 ]]; then
    printf 'pin hygiene check: failed with %d error(s)\n' "$error_count" >&2
    exit 1
  fi

  printf 'pin hygiene check: passed\n'
}

main "$@"
