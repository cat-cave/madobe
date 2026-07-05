# shellcheck shell=bash
# shellcheck disable=SC1091,SC2154
# shellcheck source=scripts/qd-report-check/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

validate_blocker() {
  local file=$1
  local node_id
  local status

  node_id=$(node_id_for_report "$file")

  validate_blocker_path "$file" "$node_id"
  validate_blocker_node_exists "$file" "$node_id" || return
  validate_blocker_mentions_node "$file" "$node_id"

  status=$(blocker_node_status "$node_id")
  case "$status" in
  blocked)
    validate_active_blocker "$file" "$node_id"
    ;;
  done)
    validate_historical_blocker "$file"
    ;;
  *)
    report_error "$file" "blocker reports are only valid for blocked nodes or done nodes with historical context: node status is $status"
    ;;
  esac
}

validate_blocker_path() {
  local file=$1
  local node_id=$2

  if [[ $file != reports/qd/"$node_id"/blocker.md ]]; then
    report_error "$file" "blocker report path must be reports/qd/<node-id>/blocker.md"
  fi
}

validate_blocker_node_exists() {
  local file=$1
  local node_id=$2

  if [[ ! -f $roadmap_export ]]; then
    report_error "$file" "cannot validate blocker node id without $roadmap_export"
    return 1
  fi

  if ! jq -e 'type == "object" and (.nodes | type == "array")' "$roadmap_export" >/dev/null 2>&1; then
    report_error "$file" "cannot validate blocker node id because $roadmap_export is not a qd export with nodes"
    return 1
  fi

  if ! node_exists_in_roadmap "$node_id"; then
    report_error "$file" "blocker report directory must match a node id in $roadmap_export"
    return 1
  fi
}

blocker_node_status() {
  local node_id=$1

  jq -r --arg node_id "$node_id" '.nodes[] | select(.id == $node_id) | .status' "$roadmap_export" | head -n 1
}

validate_blocker_mentions_node() {
  local file=$1
  local node_id=$2

  if ! grep -Fq -- "$node_id" "$file"; then
    report_error "$file" "blocker report must mention its node id: $node_id"
  fi
}

validate_active_blocker() {
  local file=$1
  local node_id=$2

  require_markdown_token "$file" '(^|[[:space:]])platform[[:space:]]*:' "active blocker must include platform"
  require_markdown_token "$file" '(^|[[:space:]])(command|operation)[[:space:]]*:' "active blocker must include failed command or operation"
  require_markdown_token "$file" '(^|[[:space:]])missing condition[[:space:]]*:' "active blocker must include missing condition"
  require_markdown_token "$file" '(^|[[:space:]])evidence[[:space:]]*:' "active blocker must include evidence path section"
  require_markdown_token "$file" '(^|[[:space:]])unblock path[[:space:]]*:' "active blocker must include unblock path"
  validate_blocker_evidence_paths "$file" "$node_id"
}

validate_historical_blocker() {
  local file=$1

  require_markdown_token "$file" '(^|[[:space:]])(superseded|historical)[[:space:]]*:' "done-node blocker must be explicitly marked Superseded: or Historical:"
}

require_markdown_token() {
  local file=$1
  local pattern=$2
  local message=$3

  if ! grep -Eiq -- "$pattern" "$file"; then
    report_error "$file" "$message"
  fi
}

validate_blocker_evidence_paths() {
  local file=$1
  local node_id=$2
  local path
  local found=0

  while IFS= read -r path; do
    found=1
    if ! validate_repo_relative_evidence_path "$file" "$path"; then
      continue
    fi
    if [[ ! -e $path ]]; then
      report_error "$file" "evidence path does not exist: $path"
    fi
  done < <(extract_blocker_evidence_paths "$file" "$node_id")

  if [[ $found -eq 0 ]]; then
    report_error "$file" "active blocker must include at least one repo-relative evidence path under evidence/<node-id>/"
  fi
}

extract_blocker_evidence_paths() {
  local file=$1
  local node_id=$2

  grep -Eo "evidence/${node_id}/[^][()[:space:]'\"<>]+" "$file" | sed 's/[`,.;:]$//' | sort -u
}
