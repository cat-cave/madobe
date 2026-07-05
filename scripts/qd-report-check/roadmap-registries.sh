# shellcheck shell=bash
# shellcheck disable=SC1091,SC2154
# shellcheck source=scripts/qd-report-check/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

validate_roadmap_milestone_names() {
  local index
  local name

  while IFS=$'\t' read -r index name; do
    report_error "$roadmap_export" "registries.milestones[$index].name must be a non-blank string: $name"
  done < <(
    jq -r '
      .registries.milestones
      | to_entries[]
      | .key as $index
      | (.value.name // null) as $raw_name
      | if ($raw_name | type) != "string" then
          [$index, ($raw_name | tostring)]
        else
          ($raw_name | gsub("^[[:space:]]+|[[:space:]]+$"; "")) as $name
          | select(($name | length) == 0)
          | [$index, "<blank>"]
        end
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_milestone_ranks() {
  local index
  local rank

  while IFS=$'\t' read -r index rank; do
    report_error "$roadmap_export" "registries.milestones[$index].rank must be an integer: $rank"
  done < <(
    jq -r '
      .registries.milestones
      | to_entries[]
      | .key as $index
      | (.value.rank // null) as $rank
      | select(($rank | type) != "number" or (($rank % 1) != 0))
      | [$index, ($rank | tostring)]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_milestone_duplicate_field() {
  local field=$1
  local label=$2
  local index
  local value

  while IFS=$'\t' read -r index value; do
    report_error "$roadmap_export" "registries.milestones[$index].$field duplicates milestone $label: $value"
  done < <(
    jq -r --arg field "$field" '
      .registries.milestones
      | to_entries
      | map({
          index: .key,
          value: (
            if (.value[$field] | type) == "string" then
              (.value[$field] | gsub("^[[:space:]]+|[[:space:]]+$"; ""))
            elif (.value[$field] | type) == "number" and ((.value[$field] % 1) == 0) then
              (.value[$field] | tostring)
            else
              ""
            end
          )
        })
      | sort_by(.value)
      | group_by(.value)[]
      | select(.[0].value != "" and length > 1)
      | .[1:][]
      | [.index, .value]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_milestone_created_at() {
  local created_at
  local index

  while IFS=$'\t' read -r index created_at; do
    report_error "$roadmap_export" "registries.milestones[$index].created_at must equal 1970-01-01T00:00:00.000Z: $created_at"
  done < <(
    jq -r '
      .registries.milestones
      | to_entries[]
      | .key as $index
      | (.value.created_at // null) as $created_at
      | select($created_at != "1970-01-01T00:00:00.000Z")
      | [$index, ($created_at | tostring)]
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_node_milestones() {
  local index
  local milestone

  while IFS=$'\t' read -r index milestone; do
    report_error "$roadmap_export" "nodes[$index].milestone must match a registered milestone name or null: $milestone"
  done < <(
    jq -r '
      ([.registries.milestones[].name | select(type == "string") | gsub("^[[:space:]]+|[[:space:]]+$"; "")] | INDEX(.)) as $milestones
      | .nodes
      | to_entries[]
      | .key as $index
      | (.value.milestone // null) as $raw_milestone
      | select($raw_milestone != null)
      | if ($raw_milestone | type) != "string" then
          [$index, ($raw_milestone | tostring)]
        else
          ($raw_milestone | gsub("^[[:space:]]+|[[:space:]]+$"; "")) as $milestone
          | select(($milestone | length) == 0 or ($milestones[$milestone] | not))
          | [$index, (if ($milestone | length) == 0 then "<blank>" else $milestone end)]
        end
      | @tsv
    ' "$roadmap_export"
  )
}

validate_roadmap_registries() {
  validate_roadmap_milestone_names
  validate_roadmap_milestone_duplicate_field name name
  validate_roadmap_milestone_ranks
  validate_roadmap_milestone_duplicate_field rank rank
  validate_roadmap_milestone_created_at
  validate_roadmap_node_milestones
}
