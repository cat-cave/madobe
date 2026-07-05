# shellcheck shell=bash
# shellcheck disable=SC1091,SC2154
# shellcheck source=scripts/qd-report-check/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

validate_roadmap_run_content() {
  local index
  local field
  local value
  local path

  while IFS=$'\t' read -r index value; do
    report_error "$roadmap_export" "runs[$index].id must be a lowercase UUID-shaped string: $value"
  done < <(
    jq -r '
      (.runs // [])
      | to_entries[]
      | .key as $index
      | (.value.id // null) as $id
      | select(($id | type) != "string" or (($id | test("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")) | not))
      | [$index, ($id | tostring)]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index field value; do
    report_error "$roadmap_export" "runs[$index].$field must be an ISO-like timestamp string: $value"
  done < <(
    jq -r '
      def timestamp_shape:
        type == "string"
        and (gsub("^[[:space:]]+|[[:space:]]+$"; "") | length > 0)
        and test("^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(\\.[0-9]{1,9})?(Z|[+-][0-9]{2}:[0-9]{2})$");
      (.runs // [])
      | to_entries[]
      | .key as $index
      | .value as $run
      | ["started_at", "finished_at"][]
      | . as $field
      | $run[$field] as $value
      | select(
          if $field == "finished_at" and $value == null then
            false
          else
            ($value | timestamp_shape | not)
          end
        )
      | [$index, $field, ($value | tostring)]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index field value; do
    report_error "$roadmap_export" "runs[$index].$field must be null or a string: $value"
  done < <(
    jq -r '
      (.runs // [])
      | to_entries[]
      | .key as $index
      | .value as $run
      | ["command", "provider", "external_id", "url", "rationale", "superseded_by", "audit_kind", "worktree_path", "agent", "summary", "log_path"][]
      | . as $field
      | $run[$field] as $value
      | select($value != null and ($value | type) != "string")
      | [$index, $field, ($value | tostring)]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index value; do
    report_error "$roadmap_export" "runs[$index].git_sha must be null or a 40-character lowercase hex string: $value"
  done < <(
    jq -r '
      (.runs // [])
      | to_entries[]
      | .key as $index
      | .value.git_sha as $git_sha
      | select($git_sha != null)
      | select(($git_sha | type) != "string" or (($git_sha | test("^[0-9a-f]{40}$")) | not))
      | [$index, ($git_sha | tostring)]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index value; do
    report_error "$roadmap_export" "runs[$index].exit_code must be null or an integer number: $value"
  done < <(
    jq -r '
      (.runs // [])
      | to_entries[]
      | .key as $index
      | .value.exit_code as $exit_code
      | select($exit_code != null)
      | select(($exit_code | type) != "number" or (($exit_code | floor) != $exit_code))
      | [$index, ($exit_code | tostring)]
      | @tsv
    ' "$roadmap_export"
  )

  while IFS=$'\t' read -r index path; do
    if [[ $path = /* ]]; then
      report_error "$roadmap_export" "runs[$index].log_path must be repo-relative: $path"
      continue
    fi

    if path_has_traversal "$path"; then
      report_error "$roadmap_export" "runs[$index].log_path must not contain '..' traversal: $path"
      continue
    fi

    if [[ ! -e $path ]]; then
      report_error "$roadmap_export" "runs[$index].log_path does not exist: $path"
    fi
  done < <(
    jq -r '
      (.runs // [])
      | to_entries[]
      | .key as $index
      | .value.log_path as $log_path
      | select(($log_path | type) == "string")
      | ($log_path | gsub("^[[:space:]]+|[[:space:]]+$"; "")) as $path
      | select(($path | length) > 0)
      | [$index, $path]
      | @tsv
    ' "$roadmap_export"
  )
}
