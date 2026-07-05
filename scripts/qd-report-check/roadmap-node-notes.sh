# shellcheck shell=bash
# shellcheck disable=SC1091,SC2154
# shellcheck source=scripts/qd-report-check/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

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

validate_roadmap_node_note_evidence_types() {
  local index
  local value

  while IFS=$'\t' read -r index value; do
    report_error "$roadmap_export" "node_notes[$index].evidence must be a string when present: $value"
  done < <(
    jq -r '
      (.node_notes // [])
      | to_entries[]
      | .key as $index
      | (.value.evidence // null) as $raw_evidence
      | select($raw_evidence != null and ($raw_evidence | type) != "string")
      | [$index, ($raw_evidence | tostring)]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_node_note_evidence_paths() {
  local index
  local evidence

  while IFS=$'\t' read -r index evidence; do
    if [[ -z $evidence ]]; then
      report_error "$roadmap_export" "node_notes[$index].evidence must be a non-blank string when present"
      continue
    fi

    if [[ $evidence =~ ^https?:// ]]; then
      continue
    fi

    if [[ $evidence = /* ]]; then
      report_error "$roadmap_export" "node_notes[$index].evidence must be repo-relative or HTTP(S): $evidence"
      continue
    fi

    if path_has_traversal "$evidence"; then
      report_error "$roadmap_export" "node_notes[$index].evidence must not contain '..' traversal: $evidence"
      continue
    fi

    if [[ ! -e $evidence ]]; then
      report_error "$roadmap_export" "node_notes[$index].evidence does not exist: $evidence"
    fi
  done < <(
    jq -r '
      (.node_notes // [])
      | to_entries[]
      | .key as $index
      | (.value.evidence // null) as $raw_evidence
      | select($raw_evidence != null and ($raw_evidence | type) == "string")
      | [$index, ($raw_evidence | gsub("^[[:space:]]+|[[:space:]]+$"; ""))]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_node_notes() {
  validate_roadmap_node_note_node_ids
  validate_roadmap_record_values node_notes kind '["blocker","retry","note"]' "blocker, retry, note"
  validate_roadmap_node_note_required_strings id
  validate_roadmap_node_note_required_strings text
  validate_roadmap_node_note_duplicate_ids
  validate_roadmap_node_note_evidence_types
  validate_roadmap_node_note_evidence_paths
}
