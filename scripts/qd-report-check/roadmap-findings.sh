# shellcheck shell=bash
# shellcheck disable=SC1091,SC2154
# shellcheck source=scripts/qd-report-check/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

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

validate_roadmap_finding_content() {
  local index
  local field
  local value
  local path

  while IFS=$'\t' read -r index value; do
    report_error "$roadmap_export" "findings[$index].id must be a lowercase UUID-shaped string: $value"
  done < <(
    jq -r '
      (.findings // [])
      | to_entries[]
      | .key as $index
      | (.value.id // null) as $id
      | select(($id | type) != "string" or (($id | test("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")) | not))
      | [$index, ($id | tostring)]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index field value; do
    report_error "$roadmap_export" "findings[$index].$field must be a non-blank string: $value"
  done < <(
    jq -r '
      (.findings // [])
      | to_entries[]
      | .key as $index
      | .value as $finding
      | ["title", "evidence"][]
      | . as $field
      | $finding[$field] as $value
      | select(($value | type) != "string" or (($value | gsub("^[[:space:]]+|[[:space:]]+$"; "") | length) == 0))
      | [$index, $field, ($value | tostring)]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index value; do
    report_error "$roadmap_export" "findings[$index].path must be null or a string: $value"
  done < <(
    jq -r '
      (.findings // [])
      | to_entries[]
      | .key as $index
      | .value.path as $path
      | select($path != null and ($path | type) != "string")
      | [$index, ($path | tostring)]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index path; do
    if [[ -z $path ]]; then
      report_error "$roadmap_export" "findings[$index].path must not be blank when present"
      continue
    fi

    if [[ $path = /* ]]; then
      report_error "$roadmap_export" "findings[$index].path must be repo-relative: $path"
      continue
    fi

    if path_has_traversal "$path"; then
      report_error "$roadmap_export" "findings[$index].path must not contain '..' traversal: $path"
    fi
  done < <(
    jq -r '
      (.findings // [])
      | to_entries[]
      | .key as $index
      | .value.path as $path
      | select($path != null and ($path | type) == "string")
      | [$index, ($path | gsub("^[[:space:]]+|[[:space:]]+$"; ""))]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index value; do
    report_error "$roadmap_export" "findings[$index].line must be a positive integer when present: $value"
  done < <(
    jq -r '
      (.findings // [])
      | to_entries[]
      | .key as $index
      | .value.line as $line
      | select($line != null)
      | select(($line | type) != "number" or (($line % 1) != 0) or $line <= 0)
      | [$index, ($line | tostring)]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index field value; do
    report_error "$roadmap_export" "findings[$index].$field must be null or a string: $value"
  done < <(
    jq -r '
      (.findings // [])
      | to_entries[]
      | .key as $index
      | .value as $finding
      | ["expected", "suggested_fix"][]
      | . as $field
      | $finding[$field] as $value
      | select($value != null and ($value | type) != "string")
      | [$index, $field, ($value | tostring)]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index field value; do
    report_error "$roadmap_export" "findings[$index].$field must be an ISO-like timestamp string: $value"
  done < <(
    jq -r '
      def timestamp_shape:
        type == "string"
        and (gsub("^[[:space:]]+|[[:space:]]+$"; "") | length > 0)
        and test("^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(\\.[0-9]{1,9})?(Z|[+-][0-9]{2}:[0-9]{2})$");
      (.findings // [])
      | to_entries[]
      | .key as $index
      | .value as $finding
      | ["created_at", "resolved_at"][]
      | . as $field
      | $finding[$field] as $value
      | select(
          if $field == "resolved_at" and $value == null then
            false
          else
            ($value | timestamp_shape | not)
          end
        )
      | [$index, $field, ($value | tostring)]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_findings() {
  validate_roadmap_finding_node_ids
  validate_roadmap_record_values findings severity '["P0","P1","P2","P3"]' "P0, P1, P2, P3"
  validate_roadmap_record_values findings status '["open","resolved","promoted"]' "open, resolved, promoted"
  validate_roadmap_finding_content
}
