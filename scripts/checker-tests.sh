#!/usr/bin/env bash
# Regression tests for the repo contract-check scripts.
#
# Each case runs one checker against either the committed repo state (positive
# cases) or a mutated/fixture copy (negative cases) and asserts the checker
# exit status plus, for negative cases, the specific diagnostic message that
# must appear on stderr.
set -euo pipefail

script_dir=$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(CDPATH='' cd -- "$script_dir/.." && pwd)

pass_count=0
fail_count=0
work_dir=''

setup_work_dir() {
  work_dir=$(mktemp -d "${TMPDIR:-/tmp}/madobe-checker-tests.XXXXXX")
  trap 'rm -rf "$work_dir"' EXIT
}

record_pass() {
  printf 'checker tests: pass: %s\n' "$1"
  pass_count=$((pass_count + 1))
}

record_fail() {
  printf 'checker tests: FAIL: %s: %s\n' "$1" "$2" >&2
  fail_count=$((fail_count + 1))
}

expect_pass() {
  local label=$1
  shift

  if "$@" >/dev/null 2>"$work_dir/stderr.log"; then
    record_pass "$label"
  else
    record_fail "$label" "expected success, got failure: $(cat "$work_dir/stderr.log")"
  fi
}

expect_fail_with_message() {
  local label=$1
  local expected_message=$2
  shift 2

  if "$@" >/dev/null 2>"$work_dir/stderr.log"; then
    record_fail "$label" "expected failure, got success"
    return
  fi

  if grep -Fq -- "$expected_message" "$work_dir/stderr.log"; then
    record_pass "$label"
  else
    record_fail "$label" "expected diagnostic not found: $expected_message"
  fi
}

mutated_copy() {
  local source_file=$1
  local mutation=$2
  local target_file=$3

  sed -E "$mutation" "$source_file" >"$target_file"
}

deleted_line_copy() {
  local source_file=$1
  local pattern=$2
  local target_file=$3

  sed "/$pattern/d" "$source_file" >"$target_file"
}

pin_hygiene_in() {
  local directory=$1

  (
    cd "$directory" || exit 1
    exec bash "$script_dir/pin-hygiene-check.sh"
  )
}

workflow_contract_cases() {
  local checker="$script_dir/workflow-contract-check.sh"
  local ci_file="$repo_root/.github/workflows/ci.yml"
  local nightly_file="$repo_root/.github/workflows/nightly.yml"

  expect_pass \
    "workflow contract accepts committed workflows" \
    bash "$checker"

  mutated_copy "$ci_file" \
    's|(uses: actions/checkout)@[0-9A-Fa-f]{40}|\1@v9|' \
    "$work_dir/ci-tag-pin.yml"
  expect_fail_with_message \
    "workflow contract rejects tag-pinned external action" \
    "external actions must be pinned to full 40-character commit SHAs" \
    bash "$checker" "$work_dir/ci-tag-pin.yml" "$nightly_file"

  mutated_copy "$ci_file" \
    's|persist-credentials: false|persist-credentials: true|' \
    "$work_dir/ci-persist-creds.yml"
  expect_fail_with_message \
    "workflow contract rejects checkout without persist-credentials false" \
    "actions/checkout must set persist-credentials: false" \
    bash "$checker" "$work_dir/ci-persist-creds.yml" "$nightly_file"

  deleted_line_copy "$ci_file" \
    'timeout-minutes: 45' \
    "$work_dir/ci-missing-timeout.yml"
  expect_fail_with_message \
    "workflow contract rejects job without timeout-minutes" \
    "missing timeout-minutes in linux job" \
    bash "$checker" "$work_dir/ci-missing-timeout.yml" "$nightly_file"
}

pin_hygiene_cases() {
  expect_pass \
    "pin hygiene accepts committed repo state" \
    pin_hygiene_in "$repo_root"

  mkdir -p "$work_dir/tag-pin-tree/.github/workflows"
  printf '%s\n' \
    'name: ci' \
    'on: [push]' \
    'jobs:' \
    '  build:' \
    '    runs-on: ubuntu-latest' \
    '    steps:' \
    '      - uses: actions/checkout@v9' \
    >"$work_dir/tag-pin-tree/.github/workflows/ci.yml"
  expect_fail_with_message \
    "pin hygiene rejects tag-pinned action" \
    "pin external action actions/checkout@v9 to a full 40-character commit SHA" \
    pin_hygiene_in "$work_dir/tag-pin-tree"

  mkdir -p "$work_dir/latest-tool-tree"
  printf '%s\n' \
    '[tools]' \
    'ruby = "latest"' \
    >"$work_dir/latest-tool-tree/.mise.toml"
  expect_fail_with_message \
    "pin hygiene rejects latest tool selector" \
    "pin native tool selector instead of using latest" \
    pin_hygiene_in "$work_dir/latest-tool-tree"
}

dependabot_contract_cases() {
  local checker="$script_dir/dependabot-contract-check.sh"

  printf '%s\n' \
    'version: 2' \
    'updates:' \
    '  - package-ecosystem: "github-actions"' \
    '    directory: "/"' \
    '    schedule:' \
    '      interval: "weekly"' \
    '  - package-ecosystem: "cargo"' \
    '    directory: "/"' \
    '    schedule:' \
    '      interval: "weekly"' \
    >"$work_dir/dependabot-valid.yml"
  expect_pass \
    "dependabot contract accepts minimal weekly github-actions and cargo config" \
    bash "$checker" "$work_dir/dependabot-valid.yml"

  printf '%s\n' \
    'version: 2' \
    'updates:' \
    '  - package-ecosystem: "github-actions"' \
    '    directory: "/"' \
    '    schedule:' \
    '      interval: "weekly"' \
    >"$work_dir/dependabot-no-cargo.yml"
  expect_fail_with_message \
    "dependabot contract rejects missing cargo update entry" \
    "missing weekly cargo update entry rooted at /" \
    bash "$checker" "$work_dir/dependabot-no-cargo.yml"
}

pr_template_cases() {
  local checker="$script_dir/pr-template-check.sh"
  local template_file="$repo_root/.github/PULL_REQUEST_TEMPLATE.md"

  expect_pass \
    "pr template check accepts committed template" \
    bash "$checker" "$template_file"

  deleted_line_copy "$template_file" \
    '^## Verification$' \
    "$work_dir/template-missing-verification.md"
  expect_fail_with_message \
    "pr template check rejects missing verification heading" \
    "missing verification heading" \
    bash "$checker" "$work_dir/template-missing-verification.md"
}

main() {
  setup_work_dir

  workflow_contract_cases
  pin_hygiene_cases
  dependabot_contract_cases
  pr_template_cases

  if [[ $fail_count -ne 0 ]]; then
    printf 'checker tests: failed with %d failure(s), %d case(s) passed\n' \
      "$fail_count" "$pass_count" >&2
    exit 1
  fi

  printf 'checker tests: %d case(s) passed\n' "$pass_count"
}

main "$@"
