#!/usr/bin/env bash
# shellcheck disable=SC1091,SC2154
set -euo pipefail

script_dir=$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

# shellcheck source=scripts/qd-report-check/common.sh
source "$script_dir/qd-report-check/common.sh"
# shellcheck source=scripts/qd-report-check/completion.sh
source "$script_dir/qd-report-check/completion.sh"
# shellcheck source=scripts/qd-report-check/audit.sh
source "$script_dir/qd-report-check/audit.sh"
# shellcheck source=scripts/qd-report-check/blocker.sh
source "$script_dir/qd-report-check/blocker.sh"
# shellcheck source=scripts/qd-report-check/line-budget.sh
source "$script_dir/qd-report-check/line-budget.sh"
# shellcheck source=scripts/qd-report-check/roadmap.sh
source "$script_dir/qd-report-check/roadmap.sh"

main() {
  local completion_count=0
  local audit_count=0
  local blocker_count=0
  local file
  local roadmap_valid=0

  command -v jq >/dev/null 2>&1 || {
    printf 'qd report check: required tool missing: jq\n' >&2
    exit 127
  }

  validate_qd_report_check_line_budget

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

  while IFS= read -r -d '' file; do
    blocker_count=$((blocker_count + 1))
    validate_blocker "$file"
  done < <(find reports/qd -mindepth 2 -maxdepth 2 -type f -name blocker.md -print0 | sort -z)

  if [[ $completion_count -eq 0 ]]; then
    report_error reports/qd "no completion reports found"
  fi
  if [[ $audit_count -eq 0 ]]; then
    report_error reports/qd "no audit reports found"
  fi

  if [[ $roadmap_valid -eq 1 ]]; then
    validate_roadmap_runs
    validate_roadmap_findings
    validate_roadmap_run_report_paths
    validate_report_directory_coverage
    validate_done_node_report_coverage
  fi

  if [[ $error_count -ne 0 ]]; then
    printf 'qd report check: failed with %d error(s)\n' "$error_count" >&2
    exit 1
  fi

  printf 'qd report check: validated %d completion report(s), %d audit report(s), and %d blocker report(s)\n' "$completion_count" "$audit_count" "$blocker_count"
}

main "$@"
