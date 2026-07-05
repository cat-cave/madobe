# shellcheck shell=bash
# shellcheck disable=SC1091,SC2154
# shellcheck source=scripts/qd-report-check/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

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
