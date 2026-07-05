# shellcheck shell=bash
# shellcheck disable=SC1091,SC2154
# shellcheck source=scripts/qd-report-check/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

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
    and all(["nodes", "edges", "runs", "findings", "node_notes", "assignments", "waves", "wave_memberships"][]; ($export[.] | type) == "array")
  ' "$roadmap_export" >/dev/null; then
    report_error "$roadmap_export" "must use deterministic qd export envelope: schema_version 2, epoch exported_at, object registries, and array top-level collections"
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
}

validate_roadmap_finding_node_ids() {
  local index
  local node_id

  while IFS=$'\t' read -r index node_id; do
    report_error "$roadmap_export" "findings[$index].node_id must be a non-blank existing node id: $node_id"
  done < <(
    jq -r '
      ([.nodes[].id] | INDEX(.)) as $node_ids
      | (.findings // [])
      | to_entries[]
      | .key as $index
      | (.value.node_id // null) as $raw_node_id
      | if ($raw_node_id | type) != "string" then
          [$index, ($raw_node_id | tostring)]
        else
          ($raw_node_id | gsub("^[[:space:]]+|[[:space:]]+$"; "")) as $node_id
          | select(($node_id | length) == 0 or ($node_ids[$node_id] | not))
          | [$index, (if ($node_id | length) == 0 then "<blank>" else $node_id end)]
        end
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_findings() {
  validate_roadmap_finding_node_ids
  validate_roadmap_record_values findings severity '["P0","P1","P2","P3"]' "P0, P1, P2, P3"
  validate_roadmap_record_values findings status '["open","resolved","promoted"]' "open, resolved, promoted"
}

validate_roadmap_node_note_node_ids() {
  local index
  local node_id

  while IFS=$'\t' read -r index node_id; do
    report_error "$roadmap_export" "node_notes[$index].node_id must be a non-blank existing node id: $node_id"
  done < <(
    jq -r '
      ([.nodes[].id] | INDEX(.)) as $node_ids
      | (.node_notes // [])
      | to_entries[]
      | .key as $index
      | (.value.node_id // null) as $raw_node_id
      | if ($raw_node_id | type) != "string" then
          [$index, ($raw_node_id | tostring)]
        else
          ($raw_node_id | gsub("^[[:space:]]+|[[:space:]]+$"; "")) as $node_id
          | select(($node_id | length) == 0 or ($node_ids[$node_id] | not))
          | [$index, (if ($node_id | length) == 0 then "<blank>" else $node_id end)]
        end
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_node_note_required_strings() {
  local field=$1
  local index
  local value

  while IFS=$'\t' read -r index value; do
    report_error "$roadmap_export" "node_notes[$index].$field must be a non-blank string: $value"
  done < <(
    jq -r --arg field "$field" '
      (.node_notes // [])
      | to_entries[]
      | .key as $index
      | (.value[$field] // null) as $raw_value
      | if ($raw_value | type) != "string" then
          [$index, ($raw_value | tostring)]
        else
          ($raw_value | gsub("^[[:space:]]+|[[:space:]]+$"; "")) as $value
          | select(($value | length) == 0)
          | [$index, "<blank>"]
        end
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_node_note_duplicate_ids() {
  local id
  local index

  while IFS=$'\t' read -r index id; do
    report_error "$roadmap_export" "node_notes[$index].id duplicates note id: $id"
  done < <(
    jq -r '
      (.node_notes // [])
      | to_entries
      | map({
          index: .key,
          id: (if (.value.id | type) == "string" then (.value.id | gsub("^[[:space:]]+|[[:space:]]+$"; "")) else (.value.id | tostring) end)
        })
      | sort_by(.id)
      | group_by(.id)[]
      | select(.[0].id != "" and length > 1)
      | .[1:][]
      | [.index, .id]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_node_notes() {
  validate_roadmap_node_note_node_ids
  validate_roadmap_record_values node_notes kind '["blocker","retry"]' "blocker, retry"
  validate_roadmap_node_note_required_strings id
  validate_roadmap_node_note_required_strings text
  validate_roadmap_node_note_duplicate_ids
}

validate_roadmap_edge_node_ids() {
  local field=$1
  local index
  local node_id

  while IFS=$'\t' read -r index node_id; do
    report_error "$roadmap_export" "edges[$index].$field must be a non-blank existing node id: $node_id"
  done < <(
    jq -r --arg field "$field" '
      ([.nodes[].id] | INDEX(.)) as $node_ids
      | (.edges // [])
      | to_entries[]
      | .key as $index
      | (.value[$field] // null) as $raw_node_id
      | if ($raw_node_id | type) != "string" then
          [$index, ($raw_node_id | tostring)]
        else
          ($raw_node_id | gsub("^[[:space:]]+|[[:space:]]+$"; "")) as $node_id
          | select(($node_id | length) == 0 or ($node_ids[$node_id] | not))
          | [$index, (if ($node_id | length) == 0 then "<blank>" else $node_id end)]
        end
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_edge_self_edges() {
  local from_node
  local index
  local to_node

  while IFS=$'\t' read -r index from_node to_node; do
    report_error "$roadmap_export" "edges[$index] must not be a self-edge: $from_node -> $to_node"
  done < <(
    jq -r '
      (.edges // [])
      | to_entries[]
      | .key as $index
      | (.value.from_node // null) as $raw_from_node
      | (.value.to_node // null) as $raw_to_node
      | select(($raw_from_node | type) == "string" and ($raw_to_node | type) == "string")
      | ($raw_from_node | gsub("^[[:space:]]+|[[:space:]]+$"; "")) as $from_node
      | ($raw_to_node | gsub("^[[:space:]]+|[[:space:]]+$"; "")) as $to_node
      | select(($from_node | length) > 0 and $from_node == $to_node)
      | [$index, $from_node, $to_node]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_edge_duplicates() {
  local from_node
  local index
  local to_node
  local type

  while IFS=$'\t' read -r index from_node to_node type; do
    report_error "$roadmap_export" "edges[$index] duplicates edge triple: $from_node -> $to_node ($type)"
  done < <(
    jq -r '
      (.edges // [])
      | to_entries
      | map({
          index: .key,
          from_node: (.value.from_node | tostring),
          to_node: (.value.to_node | tostring),
          type: (.value.type | tostring),
          key: [(.value.from_node | tostring), (.value.to_node | tostring), (.value.type | tostring)]
        })
      | sort_by(.key)
      | group_by(.key)[]
      | select(length > 1)
      | .[1:][]
      | [.index, .from_node, .to_node, .type]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_edges() {
  validate_roadmap_edge_node_ids from_node
  validate_roadmap_edge_node_ids to_node
  validate_roadmap_edge_self_edges
  validate_roadmap_edge_duplicates
  validate_roadmap_record_values edges type '["requires"]' "requires"
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
