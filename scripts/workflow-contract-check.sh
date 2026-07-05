#!/usr/bin/env bash
set -euo pipefail

error_count=0
ci_file=${1:-.github/workflows/ci.yml}
nightly_file=${2:-.github/workflows/nightly.yml}

report_error() {
  printf 'workflow contract check: %s: %s\n' "$1" "$2" >&2
  error_count=$((error_count + 1))
}

require_file() {
  local file=$1

  if [[ ! -f $file ]]; then
    report_error "$file" "required workflow file is missing"
    return 1
  fi
}

require_text() {
  local file=$1
  local expected=$2
  local description=$3

  if ! grep -Fq -- "$expected" "$file"; then
    report_error "$file" "missing $description: $expected"
  fi
}

require_pattern() {
  local file=$1
  local pattern=$2
  local description=$3

  if ! grep -Eq -- "$pattern" "$file"; then
    report_error "$file" "missing $description"
  fi
}

require_top_level_permission() {
  local file=$1
  local permission=$2

  if ! awk -v permission="$permission" '
    /^permissions:[[:space:]]*$/ {
      in_permissions = 1
      next
    }
    in_permissions && /^[[:alnum:]_.-]+:/ {
      exit
    }
    in_permissions && $0 == "  " permission {
      found = 1
      exit
    }
    END {
      exit found ? 0 : 1
    }
  ' "$file"; then
    report_error "$file" "missing top-level permissions entry: $permission"
  fi
}

require_job_pattern() {
  local file=$1
  local job=$2
  local pattern=$3
  local description=$4

  if ! awk -v job="$job" -v pattern="$pattern" '
    $0 ~ "^  " job ":[[:space:]]*$" {
      in_job = 1
      next
    }
    in_job && $0 ~ "^  [[:alnum:]_-]+:[[:space:]]*$" {
      exit
    }
    in_job && $0 ~ pattern {
      found = 1
      exit
    }
    END {
      exit found ? 0 : 1
    }
  ' "$file"; then
    report_error "$file" "missing $description in $job job"
  fi
}

require_all_jobs_have_timeout() {
  local file=$1
  local missing_jobs

  missing_jobs=$(
    awk '
      /^jobs:[[:space:]]*$/ {
        in_jobs = 1
        next
      }
      in_jobs && /^[[:alnum:]_.-]+:/ {
        if (job != "" && timeout == 0) {
          print job
        }
        exit
      }
      in_jobs && /^  [[:alnum:]_-]+:[[:space:]]*$/ {
        if (job != "" && timeout == 0) {
          print job
        }
        job = $1
        sub(/:$/, "", job)
        timeout = 0
        next
      }
      in_jobs && job != "" && /^    timeout-minutes:[[:space:]]*[0-9]+[[:space:]]*$/ {
        timeout = 1
      }
      END {
        if (job != "" && timeout == 0) {
          print job
        }
      }
    ' "$file"
  )

  if [[ -n $missing_jobs ]]; then
    while IFS= read -r job; do
      report_error "$file" "missing timeout-minutes in $job job"
    done <<<"$missing_jobs"
  fi
}

require_external_actions_pinned_to_sha() {
  local file=$1

  if ! awk '
    function trim(value) {
      sub(/^[[:space:]]+/, "", value)
      sub(/[[:space:]]+$/, "", value)
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
    {
      line = $0
      sub(/[[:space:]]+#.*/, "", line)
      if (line ~ /^[[:space:]-]*uses:[[:space:]]*.+$/) {
        uses_value = line
        sub(/^[[:space:]-]*uses:[[:space:]]*/, "", uses_value)
        uses_value = strip_quotes(trim(uses_value))
        if (uses_value ~ /^\.?\.?\// || uses_value ~ /^docker:\/\//) {
          next
        }
        if (uses_value !~ /@[0-9A-Fa-f]{40}$/) {
          print FILENAME ":" FNR ": " uses_value
          failed = 1
        }
      }
    }
    END {
      exit failed ? 1 : 0
    }
  ' "$file"; then
    report_error "$file" "external actions must be pinned to full 40-character commit SHAs"
  fi
}

require_checkout_persist_credentials_false() {
  local file=$1
  local missing_locations

  missing_locations=$(
    awk '
      function flush_checkout() {
        if (checkout_line != 0 && persist_false == 0) {
          print FILENAME ":" checkout_line
        }
      }
      /^[[:space:]-]*uses:[[:space:]]*actions\/checkout@/ {
        flush_checkout()
        checkout_line = FNR
        persist_false = 0
        next
      }
      checkout_line != 0 && /^[[:space:]-]*uses:[[:space:]]*/ {
        flush_checkout()
        checkout_line = 0
        persist_false = 0
        next
      }
      checkout_line != 0 && /^[[:space:]]*persist-credentials:[[:space:]]*false[[:space:]]*$/ {
        persist_false = 1
      }
      END {
        flush_checkout()
      }
    ' "$file"
  )

  if [[ -n $missing_locations ]]; then
    while IFS= read -r location; do
      report_error "$location" "actions/checkout must set persist-credentials: false"
    done <<<"$missing_locations"
  fi
}

check_ci_workflow() {
  require_file "$ci_file" || return

  require_pattern "$ci_file" '^  pull_request:[[:space:]]*$' "pull_request trigger"
  require_pattern "$ci_file" '^  push:[[:space:]]*$' "push trigger"
  require_pattern "$ci_file" '^    branches:[[:space:]]*\[main\][[:space:]]*$' "push to main branch filter"
  require_pattern "$ci_file" '^  merge_group:[[:space:]]*$' "merge_group trigger"

  require_top_level_permission "$ci_file" "contents: read"

  require_pattern "$ci_file" '^  linux:[[:space:]]*$' "linux job"
  require_pattern "$ci_file" '^  macos:[[:space:]]*$' "macos job"
  require_job_pattern "$ci_file" "linux" '^[[:space:]]+runs-on:[[:space:]]+ubuntu-' "Linux runner"
  require_job_pattern "$ci_file" "macos" '^[[:space:]]+runs-on:[[:space:]]+macos-' "macOS runner"
  require_all_jobs_have_timeout "$ci_file"
  require_external_actions_pinned_to_sha "$ci_file"
  require_checkout_persist_credentials_false "$ci_file"

  require_text "$ci_file" "uses: actions/checkout@" "actions/checkout action"
  require_text "$ci_file" "fetch-depth: 0" "full-history checkout"
  require_text "$ci_file" "uses: cachix/install-nix-action@" "cachix/install-nix-action action"
  require_text "$ci_file" "uses: jdx/mise-action@" "jdx/mise-action action"
  require_text "$ci_file" "rustup component add rustfmt clippy" "rustfmt and clippy component install"

  require_text "$ci_file" "run: nix flake check" "nix flake check command"
  require_text "$ci_file" "run: nix develop -c just direct-capture-preflight" "direct-capture preflight command"
  require_text "$ci_file" "run: nix develop -c just check" "just check command"
  require_text "$ci_file" "run: nix develop -c just test" "just test command"
  require_text "$ci_file" "run: nix develop -c just security" "just security command"
  require_text "$ci_file" "run: just macos-check" "macOS check command"
}

check_nightly_workflow() {
  require_file "$nightly_file" || return

  require_pattern "$nightly_file" '^  schedule:[[:space:]]*$' "scheduled trigger"
  require_pattern "$nightly_file" '^[[:space:]]+- cron:' "cron schedule"
  require_pattern "$nightly_file" '^  workflow_dispatch:[[:space:]]*$' "workflow_dispatch trigger"

  require_top_level_permission "$nightly_file" "contents: read"
  require_top_level_permission "$nightly_file" "security-events: write"

  require_pattern "$nightly_file" '^  deep-checks:[[:space:]]*$' "deep-checks job"
  require_all_jobs_have_timeout "$nightly_file"
  require_external_actions_pinned_to_sha "$nightly_file"
  require_checkout_persist_credentials_false "$nightly_file"

  require_text "$nightly_file" "run: nix develop -c just security" "just security command"
  require_text "$nightly_file" "run: nix develop -c just coverage" "just coverage command"
  require_text "$nightly_file" "run: nix develop -c just mutants" "just mutants command"
  require_text "$nightly_file" "run: nix develop -c lychee ." "lychee command"
  require_text "$nightly_file" "run: nix develop -c zizmor .github/workflows" "zizmor command"
  require_text "$nightly_file" "run: nix develop -c deadnix --fail ." "deadnix command"
  require_text "$nightly_file" "run: nix develop -c statix check ." "statix command"
}

main() {
  check_ci_workflow
  check_nightly_workflow

  if [[ $error_count -ne 0 ]]; then
    printf 'workflow contract check: failed with %d error(s)\n' "$error_count" >&2
    exit 1
  fi

  printf 'workflow contract check: passed\n'
}

main "$@"
