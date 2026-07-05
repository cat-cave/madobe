#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/qd-export-parity-check.sh [--qd-root <path>] [--expected <path>] [--keep-temp]

Compares a fresh deterministic qd export from --qd-root with the expected
committed export. By default, both are the current checkout and
roadmap/qd-export.json.
USAGE
}

qd_root=$(pwd)
expected=roadmap/qd-export.json
keep_temp=0

while [[ $# -gt 0 ]]; do
  case "$1" in
  --qd-root)
    qd_root=${2:?--qd-root requires a path}
    shift 2
    ;;
  --expected)
    expected=${2:?--expected requires a path}
    shift 2
    ;;
  --keep-temp)
    keep_temp=1
    shift
    ;;
  -h | --help)
    usage
    exit 0
    ;;
  *)
    printf 'unknown argument: %s\n' "$1" >&2
    usage >&2
    exit 2
    ;;
  esac
done

if [[ ! -d $qd_root ]]; then
  printf 'qd export parity check: qd root does not exist: %s\n' "$qd_root" >&2
  exit 1
fi

qd_root=$(CDPATH='' cd -- "$qd_root" && pwd)
if [[ $expected != /* ]]; then
  expected=$qd_root/$expected
fi

if [[ ! -f $expected ]]; then
  printf 'qd export parity check: expected export does not exist: %s\n' "$expected" >&2
  exit 1
fi

tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/madobe-qd-export-parity.XXXXXX")
trap '
  if [[ $keep_temp -eq 0 ]]; then
    rm -rf "$tmp_dir"
  else
    printf "qd export parity check: kept temp dir: %s\n" "$tmp_dir" >&2
  fi
' EXIT

actual=$tmp_dir/qd-export.actual.json

(
  cd "$qd_root"
  qd export --out "$actual" --deterministic >/dev/null
)

if cmp -s "$expected" "$actual"; then
  printf 'qd export parity check: live qd export matches %s\n' "$expected"
  exit 0
fi

printf 'qd export parity check: live qd export differs from %s\n' "$expected" >&2
diff -u "$expected" "$actual" >&2 || true
exit 1
