#!/usr/bin/env bash
set -euo pipefail

script_dir=$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
repo_root=$(git -C "$script_dir/.." rev-parse --show-toplevel 2>/dev/null || pwd)
default_export="$repo_root/roadmap/qd-export.json"
check_name="qd ready queue check"
cycle_smoke_tmpdir=

usage() {
  cat <<'USAGE'
usage:
  scripts/qd-ready-queue-check.sh [roadmap-export.json]
  scripts/qd-ready-queue-check.sh --cycle-smoke [roadmap-export.json]

Reports raw ready-status nodes from the export, assignable ready nodes whose
transitive requires prerequisites are all done, and ready-status nodes held by
non-done prerequisites. The cycle smoke uses a temporary export copy.
USAGE
}

require_jq() {
  command -v jq >/dev/null 2>&1 || {
    printf '%s: required tool missing: jq\n' "$check_name" >&2
    exit 127
  }
}

require_export() {
  local file=$1

  if [[ ! -f $file ]]; then
    printf '%s: export file missing: %s\n' "$check_name" "$file" >&2
    exit 1
  fi
}

# shellcheck disable=SC2016 # jq expands jq variables and interpolation.
cycle_jq='
def requires_edges:
  [.edges[]? | select(.type == "requires") | {from: .from_node, to: .to_node}];

def adjacency($edges):
  reduce $edges[] as $edge ({}; .[$edge.to] += [$edge.from]);

def cycle_from($adj; $id; $stack):
  if ($stack | index($id)) != null then
    $stack[($stack | index($id)):] + [$id]
  else
    first(
      ($adj[$id] // [])[] as $next
      | cycle_from($adj; $next; $stack + [$id])
    )? // empty
  end;

requires_edges as $edges
| adjacency($edges) as $adj
| first(([.nodes[].id] | sort)[] as $id | cycle_from($adj; $id; []))? // empty
| if type == "array" then join(" -> ") else empty end
'

# shellcheck disable=SC2016 # jq expands jq variables and interpolation.
report_jq='
def requires_edges:
  [.edges[]? | select(.type == "requires") | {from: .from_node, to: .to_node}];

def adjacency($edges):
  reduce $edges[] as $edge ({}; .[$edge.to] += [$edge.from]);

def normalize_text:
  tostring | gsub("[\r\n]+"; " ");

def nearest_non_done($adj; $nodes; $ids; $seen; $distance):
  if ($ids | length) == 0 then
    null
  else
    [$ids[] | select((($nodes[.] // {status: "missing"}).status // "missing") != "done")] as $bad
    | if ($bad | length) > 0 then
        ($bad | sort | .[0]) as $bad_id
        | ($nodes[$bad_id] // {id: $bad_id, title: "missing node", status: "missing"}) as $bad_node
        | {
            id: $bad_id,
            title: ($bad_node.title // ""),
            status: ($bad_node.status // "missing"),
            distance: $distance,
            blocked_by: ($bad_node.blocked_by // null),
            blocked_reason: ($bad_node.blocked_reason // null),
            status_reason: ($bad_node.status_reason // null)
          }
      else
        [$ids[] as $id | ($adj[$id] // [])[] | select(($seen | index(.)) | not)] | unique as $next
        | nearest_non_done($adj; $nodes; $next; ($seen + $next); ($distance + 1))
      end
  end;

def nearest_blocked($adj; $nodes; $ids; $seen; $distance):
  if ($ids | length) == 0 then
    null
  else
    [$ids[] | select((($nodes[.] // {status: "missing"}).status // "missing") == "blocked")] as $bad
    | if ($bad | length) > 0 then
        ($bad | sort | .[0]) as $bad_id
        | $nodes[$bad_id] as $bad_node
        | {
            id: $bad_id,
            title: ($bad_node.title // ""),
            status: ($bad_node.status // "blocked"),
            distance: $distance,
            blocked_by: ($bad_node.blocked_by // null),
            blocked_reason: ($bad_node.blocked_reason // null),
            status_reason: ($bad_node.status_reason // null)
          }
      else
        [$ids[] as $id | ($adj[$id] // [])[] | select(($seen | index(.)) | not)] | unique as $next
        | nearest_blocked($adj; $nodes; $next; ($seen + $next); ($distance + 1))
      end
  end;

def field_text($prefix; $value):
  if $value == null or $value == "" then empty else $prefix + ($value | normalize_text) end;

def blocker_text($node):
  [
    field_text("blocked_by="; $node.blocked_by),
    field_text("blocked_reason="; $node.blocked_reason),
    field_text("status_reason="; $node.status_reason)
  ] | if length == 0 then "blocker_reason=none" else join("; ") end;

def node_line($node):
  "- \($node.id) [\($node.status)] \(($node.title // "") | normalize_text)";

requires_edges as $edges
| adjacency($edges) as $adj
| INDEX(.nodes[]; .id) as $nodes
| [
    .nodes[]
    | select(.status == "ready")
    | (($adj[.id] // []) | unique) as $direct_requires
    | {
        id,
        title,
        status,
        hold: {
          nearest_non_done: nearest_non_done($adj; $nodes; $direct_requires; $direct_requires; 1),
          nearest_blocked: nearest_blocked($adj; $nodes; $direct_requires; $direct_requires; 1)
        }
      }
  ] as $ready
| ($ready | map(select(.hold.nearest_non_done == null))) as $assignable
| ($ready | map(select(.hold.nearest_non_done != null))) as $held
| "\($check_name)",
  "source: \($source)",
  "selection_source: qd ready --json is authoritative for assignment; raw export status is diagnostic",
  "raw_ready_count: \($ready | length)",
  "assignable_ready_count: \($assignable | length)",
  "held_ready_count: \($held | length)",
  "",
  "Raw ready-status nodes:",
  (if ($ready | length) == 0 then "- none" else ($ready | sort_by(.id)[] | node_line(.)) end),
  "",
  "Assignable ready nodes (all transitive requires prerequisites done):",
  (if ($assignable | length) == 0 then "- none" else ($assignable | sort_by(.id)[] | node_line(.)) end),
  "",
  "Held ready-status nodes:",
  (
    if ($held | length) == 0 then
      "- none"
    else
      $held
      | sort_by(.id)[]
      | .hold.nearest_non_done as $nearest
      | .hold.nearest_blocked as $blocked
      | if $blocked == null then
          "- \(.id) held by nearest non-done \($nearest.id) [\($nearest.status)] distance=\($nearest.distance); blocker_reason=none"
        else
          "- \(.id) held by nearest non-done \($nearest.id) [\($nearest.status)] distance=\($nearest.distance); nearest blocked \($blocked.id) [\($blocked.status)] distance=\($blocked.distance); \(blocker_text($blocked))"
        end
    end
  )
'

check_export() {
  local file=$1
  local cycle

  require_export "$file"
  cycle=$(jq -r "$cycle_jq" "$file")
  if [[ -n $cycle ]]; then
    printf '%s: requires cycle detected: %s\n' "$check_name" "$cycle" >&2
    return 1
  fi

  jq -r --arg source "$file" --arg check_name "$check_name" "$report_jq" "$file"
}

cleanup_cycle_smoke() {
  if [[ -n ${cycle_smoke_tmpdir:-} ]]; then
    rm -rf -- "$cycle_smoke_tmpdir"
  fi
}

run_cycle_smoke() {
  local input_file=$1
  local cyclic_export
  local status

  require_export "$input_file"
  cycle_smoke_tmpdir=$(mktemp -d "${TMPDIR:-/tmp}/madobe-qd-ready-cycle.XXXXXX")
  trap cleanup_cycle_smoke EXIT
  cyclic_export="$cycle_smoke_tmpdir/qd-export-cyclic.json"

  jq '
    .nodes += [
      {
        "id": "__qd_ready_cycle_a",
        "title": "cycle smoke A",
        "status": "ready"
      },
      {
        "id": "__qd_ready_cycle_b",
        "title": "cycle smoke B",
        "status": "ready"
      }
    ]
    | .edges += [
      {
        "from_node": "__qd_ready_cycle_a",
        "to_node": "__qd_ready_cycle_b",
        "type": "requires"
      },
      {
        "from_node": "__qd_ready_cycle_b",
        "to_node": "__qd_ready_cycle_a",
        "type": "requires"
      }
    ]
  ' "$input_file" >"$cyclic_export"

  set +e
  "$0" "$cyclic_export" >"$cycle_smoke_tmpdir/stdout.log" 2>"$cycle_smoke_tmpdir/stderr.log"
  status=$?
  set -e

  if [[ $status -eq 0 ]]; then
    printf '%s cycle smoke: cyclic temp export unexpectedly passed: %s\n' "$check_name" "$cyclic_export" >&2
    return 1
  fi
  if ! grep -Fq 'requires cycle detected' "$cycle_smoke_tmpdir/stderr.log"; then
    printf '%s cycle smoke: cyclic temp export failed for the wrong reason: %s\n' "$check_name" "$cyclic_export" >&2
    cat "$cycle_smoke_tmpdir/stderr.log" >&2
    return 1
  fi

  printf '%s cycle smoke: rejected temp-copy cycle at %s\n' "$check_name" "$cyclic_export"
  sed 's/^/cycle smoke stderr: /' "$cycle_smoke_tmpdir/stderr.log"
}

main() {
  require_jq

  case "${1:-}" in
  -h | --help)
    usage
    ;;
  --cycle-smoke)
    run_cycle_smoke "${2:-$default_export}"
    ;;
  *)
    if [[ $# -gt 1 ]]; then
      usage >&2
      exit 2
    fi
    check_export "${1:-$default_export}"
    ;;
  esac
}

main "$@"
