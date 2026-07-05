#!/usr/bin/env bash

result_check_require_jq() {
  local check_name=$1

  command -v jq >/dev/null 2>&1 || {
    printf '%s: required tool missing: jq\n' "$check_name" >&2
    exit 127
  }
}

result_check_cd_repo_root() {
  local repo_root
  repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
  cd "$repo_root" || return
}

result_check_validate_existing_artifacts() {
  local check_name=$1
  local file=$2
  local nonempty_kinds_csv=$3
  local artifact_errors=0
  local path
  local kind

  while IFS=$'\t' read -r path kind; do
    if [[ ! -e $path ]]; then
      printf '%s: %s: artifact path does not exist: %s\n' "$check_name" "$file" "$path" >&2
      artifact_errors=$((artifact_errors + 1))
      continue
    fi

    if [[ ,$nonempty_kinds_csv, == *,$kind,* ]] && [[ ! -s $path ]]; then
      printf '%s: %s: core evidence artifact is empty: %s\n' "$check_name" "$file" "$path" >&2
      artifact_errors=$((artifact_errors + 1))
    fi
  done < <(jq -r '.artifacts[]? | [.path, .kind] | @tsv' "$file")

  [[ $artifact_errors -eq 0 ]]
}

# shellcheck disable=SC2016,SC2034 # sourced by result checker scripts; jq variables expand in jq.
result_check_common_jq_prelude='
def is_madobe_nonnegative_integer:
  type == "number" and . >= 0 and floor == .;

def is_madobe_nullable_nonnegative_integer:
  . == null or is_madobe_nonnegative_integer;

def is_madobe_repo_relative_reference:
  type == "string"
  and length > 0
  and (startswith("/") | not)
  and (test("^[A-Za-z]:[\\\\/]") | not)
  and (contains("\\") | not)
  and ((split("/") | index("..")) == null);
'
