#!/usr/bin/env bash
set -euo pipefail

error_count=0
dependabot_file=${1:-.github/dependabot.yml}

report_error() {
  printf 'dependabot contract check: %s: %s\n' "$1" "$2" >&2
  error_count=$((error_count + 1))
}

require_pattern() {
  local pattern=$1
  local description=$2

  if ! grep -Eq -- "$pattern" "$dependabot_file"; then
    report_error "$dependabot_file" "missing $description"
  fi
}

update_entry_count() {
  local ecosystem=$1
  local directory=$2
  local interval=$3

  awk -v want_ecosystem="$ecosystem" -v want_directory="$directory" -v want_interval="$interval" '
    function trim(value) {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
      return value
    }

    function strip_quotes(value) {
      if (length(value) >= 2) {
        first = substr(value, 1, 1)
        last = substr(value, length(value), 1)
        if (first == last && (first == "\"" || first == "'\''")) {
          return substr(value, 2, length(value) - 2)
        }
      }
      return value
    }

    function value_after_colon(line) {
      sub(/^[^:]+:[[:space:]]*/, "", line)
      sub(/[[:space:]]+#.*$/, "", line)
      return strip_quotes(trim(line))
    }

    function finish_entry() {
      if (in_entry && ecosystem == want_ecosystem && directory == want_directory && interval == want_interval) {
        matches++
      }
    }

    /^[[:space:]]*updates:[[:space:]]*$/ {
      in_updates = 1
      next
    }

    in_updates && /^[^[:space:]][^:]*:/ {
      finish_entry()
      in_updates = 0
      in_entry = 0
      next
    }

    in_updates && /^  -[[:space:]]+package-ecosystem:[[:space:]]*/ {
      finish_entry()
      in_entry = 1
      ecosystem = value_after_colon($0)
      directory = ""
      interval = ""
      next
    }

    in_entry && /^    directory:[[:space:]]*/ {
      directory = value_after_colon($0)
      next
    }

    in_entry && /^      interval:[[:space:]]*/ {
      interval = value_after_colon($0)
      next
    }

    END {
      finish_entry()
      print matches + 0
    }
  ' "$dependabot_file"
}

require_update_entry() {
  local ecosystem=$1
  local directory=$2
  local interval=$3
  local count

  count=$(update_entry_count "$ecosystem" "$directory" "$interval")
  if [[ $count -eq 0 ]]; then
    report_error "$dependabot_file" "missing weekly $ecosystem update entry rooted at $directory"
  fi
}

main() {
  if [[ ! -f $dependabot_file ]]; then
    report_error "$dependabot_file" "required Dependabot config is missing"
  else
    require_pattern '^version:[[:space:]]*2[[:space:]]*$' "version 2 declaration"
    require_pattern '^updates:[[:space:]]*$' "updates list"
    require_update_entry "github-actions" "/" "weekly"
    require_update_entry "cargo" "/" "weekly"
  fi

  if [[ $error_count -ne 0 ]]; then
    printf 'dependabot contract check: failed with %d error(s)\n' "$error_count" >&2
    exit 1
  fi

  printf 'dependabot contract check: passed\n'
}

main "$@"
