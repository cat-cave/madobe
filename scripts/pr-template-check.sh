#!/usr/bin/env bash
set -euo pipefail

error_count=0
template_file=${1:-.github/PULL_REQUEST_TEMPLATE.md}

report_error() {
  printf 'pr template check: %s: %s\n' "$1" "$2" >&2
  error_count=$((error_count + 1))
}

require_text() {
  local expected=$1
  local description=$2

  if ! grep -Fq -- "$expected" "$template_file"; then
    report_error "$template_file" "missing $description: $expected"
  fi
}

require_pattern() {
  local pattern=$1
  local description=$2

  if ! grep -Eq -- "$pattern" "$template_file"; then
    report_error "$template_file" "missing $description"
  fi
}

main() {
  if [[ ! -f $template_file ]]; then
    report_error "$template_file" "required PR template is missing"
  else
    require_pattern '^## qd Evidence$' "qd evidence heading"
    require_pattern '^## Verification$' "verification heading"
    require_pattern '^## Risk$' "risk heading"

    require_text '- Node id:' "qd node id field"
    require_text '- Acceptance criteria:' "acceptance criteria field"
    require_text '- Evidence paths:' "evidence paths field"
    require_text 'evidence/<node-id>/commands.log' "commands log evidence path"
    require_text 'evidence/<node-id>/notes.md' "notes evidence path"
    require_text '- qd reports:' "qd reports field"
    require_text 'reports/qd/<node-id>/completion.json' "completion report path"
    require_text 'reports/qd/<node-id>/audit.json' "audit report path"
    require_text '- Platform validation status:' "platform validation status field"
    require_text "Required platform checks: \`passed\` / \`not_required\` / \`blocked\`" "required platform checks status"
    require_text "Cross-device validation, when relevant: \`passed\` / \`not_required\` /" "cross-device validation status"
    require_text "Opposite-platform validation, when relevant: \`passed\` / \`not_required\` /" "opposite-platform validation status"
    require_text '- Residual risks:' "residual risks field"
    require_text '- DAG or topology changes:' "DAG or topology changes field"
    require_text "\`none\` or describe proposed qd node/edge changes" "DAG or topology changes guidance"
    require_text "Linux PR readiness: \`nix develop -c just ci-local\`" "Linux readiness verification"
    require_text "Native macOS validation, when relevant: \`just macos-check\`" "macOS validation verification"
    require_text 'qd evidence and reports are linked above' "qd evidence and report linkage verification"
    require_text 'DAG or topology changes are recorded above' "DAG or topology verification"
  fi

  if [[ $error_count -ne 0 ]]; then
    printf 'pr template check: failed with %d error(s)\n' "$error_count" >&2
    exit 1
  fi

  printf 'pr template check: passed\n'
}

main "$@"
