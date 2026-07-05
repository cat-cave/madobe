# shellcheck shell=bash
# shellcheck disable=SC2154

: "${qd_report_check_max_lines:=${QD_REPORT_CHECK_MAX_LINES:-350}}"

qd_report_check_line_budget_paths() {
  if [[ -n ${QD_REPORT_CHECK_LINE_BUDGET_FILES:-} ]]; then
    printf '%s\n' "$QD_REPORT_CHECK_LINE_BUDGET_FILES"
    return
  fi

  printf '%s\n' scripts/qd-reports-check.sh
  find scripts/qd-report-check -maxdepth 1 -type f -name '*.sh' -print | sort
}

validate_qd_report_check_line_budget() {
  local file
  local lines

  if [[ ! $qd_report_check_max_lines =~ ^[1-9][0-9]*$ ]]; then
    report_error "scripts/qd-report-check" "QD_REPORT_CHECK_MAX_LINES must be a positive integer: $qd_report_check_max_lines"
    return
  fi

  while IFS= read -r file; do
    [[ -z $file ]] && continue

    if [[ ! -f $file ]]; then
      report_error "$file" "line-budget path does not exist"
      continue
    fi

    lines=$(wc -l <"$file")
    lines=${lines//[[:space:]]/}

    if ((lines > qd_report_check_max_lines)); then
      report_error "$file" "has $lines lines; qd report-check line budget is $qd_report_check_max_lines"
    fi
  done < <(qd_report_check_line_budget_paths)
}
