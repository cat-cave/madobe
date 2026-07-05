# shellcheck shell=bash
# shellcheck disable=SC1091,SC2154
# shellcheck source=scripts/qd-report-check/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"
# shellcheck source=scripts/qd-report-check/roadmap-edges.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/roadmap-edges.sh"
# shellcheck source=scripts/qd-report-check/roadmap-findings.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/roadmap-findings.sh"
# shellcheck source=scripts/qd-report-check/roadmap-node-notes.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/roadmap-node-notes.sh"
# shellcheck source=scripts/qd-report-check/roadmap-registries.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/roadmap-registries.sh"
# shellcheck source=scripts/qd-report-check/roadmap-runs.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/roadmap-runs.sh"

validate_roadmap_export() {
  if [[ ! -f $roadmap_export ]]; then
    report_error "$roadmap_export" "required roadmap export is missing"
    return 1
  fi

  json_ok "$roadmap_export" || return 1

  if ! jq -e '
    . as $export
    |
    type == "object"
    and .schema_version == 2
    and .exported_at == "1970-01-01T00:00:00.000Z"
    and (.registries | type == "object")
    and all(["groups", "projects", "milestones"][]; ($export.registries[.] | type) == "array")
    and all(["nodes", "edges", "runs", "findings", "node_notes", "assignments", "waves", "wave_memberships"][]; ($export[.] | type) == "array")
  ' "$roadmap_export" >/dev/null; then
    report_error "$roadmap_export" "must use deterministic qd export envelope: schema_version 2, epoch exported_at, object registries, array registry collections, and array top-level collections"
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

  validate_roadmap_nodes
  validate_roadmap_registries
  validate_roadmap_edges
  validate_roadmap_node_notes
}

validate_roadmap_run_report_paths() {
  local path

  while IFS= read -r path; do
    if [[ $path = /* ]]; then
      report_error "$roadmap_export" "runs[].report_path must be repo-relative: $path"
      continue
    fi

    if path_has_traversal "$path"; then
      report_error "$roadmap_export" "runs[].report_path must not contain '..' traversal: $path"
      continue
    fi

    if [[ ! -e $path ]]; then
      report_error "$roadmap_export" "runs[].report_path does not exist: $path"
    fi
  done < <(
    jq -r '
      .runs[]?
      | .report_path?
      | select(type == "string")
      | select((gsub("^[[:space:]]+|[[:space:]]+$"; "") | length) > 0)
    ' "$roadmap_export"
  )
}

validate_roadmap_run_node_ids() {
  local index
  local node_id

  while IFS=$'\t' read -r index node_id; do
    report_error "$roadmap_export" "runs[$index].node_id must match an existing node id: $node_id"
  done < <(
    jq -r '
      ([.nodes[].id] | INDEX(.)) as $node_ids
      | (.runs // [])
      | to_entries[]
      | select(.value.node_id? | type == "string")
      | .key as $index
      | (.value.node_id | gsub("^[[:space:]]+|[[:space:]]+$"; "")) as $node_id
      | select(($node_id | length) > 0)
      | select($node_ids[$node_id] | not)
      | [$index, $node_id]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_record_values() {
  local collection=$1
  local field=$2
  local allowed_json=$3
  local allowed_label=$4
  local index
  local value

  while IFS=$'\t' read -r index value; do
    report_error "$roadmap_export" "${collection}[$index].$field must be one of $allowed_label: $value"
  done < <(
    jq -r --arg collection "$collection" --arg field "$field" --argjson allowed "$allowed_json" '
      (.[$collection] // [])
      | to_entries[]
      | (.value[$field]) as $value
      | select(
          ($value | type != "string")
          or (($allowed | index($value)) | not)
        )
      | [.key, ($value | tostring)]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_nodes() {
  validate_roadmap_record_values nodes status '["blocked","claimed","done","mergeable","ready","review"]' "blocked, claimed, done, mergeable, ready, review"
  validate_roadmap_record_values nodes priority '["P0","P1","P2","P3"]' "P0, P1, P2, P3"
  validate_roadmap_record_values nodes risk '["low","medium","high"]' "low, medium, high"
  validate_roadmap_record_values nodes kind '["feature","infra","docs","audit-fix"]' "feature, infra, docs, audit-fix"
}

validate_roadmap_runs() {
  validate_roadmap_run_node_ids
  validate_roadmap_record_values runs kind '["implement","audit","ci","merge"]' "implement, audit, ci, merge"
  validate_roadmap_record_values runs status '["completed","failed","passed","recorded"]' "completed, failed, passed, recorded"
  validate_roadmap_run_content
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
