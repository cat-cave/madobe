set shell := ["bash", "--noprofile", "--norc", "-eu", "-o", "pipefail", "-c"]

coverage_line_floor := "78"

default:
  just --list

require-tool tool:
  command -v "{{tool}}" >/dev/null 2>&1 || { echo "required tool missing: {{tool}}; run inside nix develop"; exit 127; }

fmt:
  cargo fmt --all -- --check
  if [ "$(uname -s)" = Darwin ] && ! command -v taplo >/dev/null 2>&1; then echo "taplo skipped outside Linux/Nix"; else just require-tool taplo; taplo fmt --check; fi
  if [ "$(uname -s)" = Darwin ] && ! command -v shfmt >/dev/null 2>&1; then echo "shfmt skipped outside Linux/Nix"; else just require-tool shfmt; find . -path './.git' -prune -o -path './target' -prune -o -name '*.sh' -print0 | xargs -0 -r shfmt -d; fi

check: fmt
  just qd-reports-check
  just product-quic-result-check
  just pin-hygiene-check
  just pr-template-check
  just workflow-contract-check
  just dependabot-contract-check
  cargo check --workspace --all-targets --all-features
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  if [ "$(uname -s)" = Darwin ] && ! command -v shellcheck >/dev/null 2>&1; then echo "shellcheck skipped outside Linux/Nix"; else just require-tool shellcheck; git ls-files -- '*.sh' | while IFS= read -r shell_file; do shellcheck "$shell_file"; done; fi
  if [ "$(uname -s)" = Darwin ] && ! command -v typos >/dev/null 2>&1; then echo "typos skipped outside Linux/Nix"; else just require-tool typos; typos; fi
  if [ "$(uname -s)" = Darwin ] && ! command -v markdownlint-cli2 >/dev/null 2>&1; then echo "markdownlint-cli2 skipped outside Linux/Nix"; else just require-tool markdownlint-cli2; markdownlint-cli2 "**/*.md"; fi
  if [ "$(uname -s)" = Darwin ] && ! command -v actionlint >/dev/null 2>&1; then echo "actionlint skipped outside Linux/Nix"; else just require-tool actionlint; actionlint .github/workflows/*.yml; fi
  just lint-lines

direct-capture-preflight:
  bash nix/direct-capture-preflight.sh

qd-reports-check:
  bash scripts/qd-reports-check.sh

product-quic-result-check:
  bash scripts/product-quic-result-check.sh

pin-hygiene-check:
  bash scripts/pin-hygiene-check.sh

pr-template-check:
  bash scripts/pr-template-check.sh

workflow-contract-check:
  bash scripts/workflow-contract-check.sh

dependabot-contract-check:
  bash scripts/dependabot-contract-check.sh

test:
  if command -v cargo-nextest >/dev/null 2>&1; then cargo nextest run --workspace --all-features; else cargo test --workspace --all-features; fi
  cargo test --doc --workspace --all-features
  just apple-test

verify: check test security

coverage:
  command -v cargo-llvm-cov >/dev/null 2>&1 || { echo "cargo-llvm-cov missing; run inside nix develop"; exit 127; }
  cargo llvm-cov nextest --workspace --all-features --ignore-filename-regex 'apps/.*/src/main\.rs' --fail-under-lines {{coverage_line_floor}} --lcov --output-path lcov.info

mutants:
  command -v cargo-mutants >/dev/null 2>&1 || { echo "cargo-mutants missing; run inside nix develop"; exit 127; }
  output_dir="$(mktemp -d "${TMPDIR:-/tmp}/madobe-mutants-smoke.XXXXXX")"; \
  trap 'rm -rf "$output_dir"' EXIT; \
  cargo mutants -p madobe-compositor --file crates/compositor/src/ids.rs --timeout 120 --jobs 2 --all-features --output "$output_dir/compositor-ids"; \
  cargo mutants -p madobe-compositor --file crates/compositor/src/status.rs --timeout 120 --jobs 2 --all-features --output "$output_dir/compositor-status"; \
  cargo mutants -p madobe-encode-nv-sys --file crates/encode-nv-sys/src/lib.rs --timeout 120 --jobs 2 --all-features --output "$output_dir/encode-nv-sys"

mutants-full:
  command -v cargo-mutants >/dev/null 2>&1 || { echo "cargo-mutants missing; run inside nix develop"; exit 127; }
  cargo mutants --workspace --timeout 120 --jobs 2

security:
  if command -v cargo-deny >/dev/null 2>&1; then cargo deny check; else echo "cargo-deny missing; run inside nix develop"; exit 127; fi
  if command -v cargo-vet >/dev/null 2>&1; then cargo vet; else echo "cargo-vet missing; run inside nix develop"; exit 127; fi
  if command -v cargo-audit >/dev/null 2>&1; then cargo audit; else echo "cargo-audit missing; run inside nix develop"; exit 127; fi
  if command -v cargo-machete >/dev/null 2>&1; then cargo machete --skip-target-dir; else echo "cargo-machete missing; run inside nix develop"; exit 127; fi
  if command -v cargo-semver-checks >/dev/null 2>&1; then cargo semver-checks check-release --workspace --baseline-root .; else echo "cargo-semver-checks missing; run inside nix develop"; exit 127; fi

ci-local:
  just direct-capture-preflight
  just verify
  just coverage

macos-bootstrap:
  if [ "$(uname -s)" != Darwin ]; then echo "macOS bootstrap skipped outside Darwin"; else if ! command -v mise >/dev/null 2>&1; then command -v brew >/dev/null 2>&1 || { echo "Homebrew missing; install jq, mise, tuist, swiftformat, and swiftlint manually"; exit 127; }; brew install mise; fi; brew list jq >/dev/null 2>&1 || brew install jq; brew list swiftformat >/dev/null 2>&1 || brew install swiftformat; brew list swiftlint >/dev/null 2>&1 || brew install swiftlint; mise install; rustup component add rustfmt clippy; fi

macos-check: check macos-swiftformat macos-swiftlint apple-generate apple-test

macos-swiftformat:
  if [ "$(uname -s)" != Darwin ]; then echo "SwiftFormat skipped outside macOS"; else command -v swiftformat >/dev/null 2>&1 || { echo "swiftformat missing; run just macos-bootstrap"; exit 127; }; swiftformat --lint apple; fi

macos-swiftlint:
  if [ "$(uname -s)" != Darwin ]; then echo "SwiftLint skipped outside macOS"; else command -v swiftlint >/dev/null 2>&1 || { echo "swiftlint missing; run just macos-bootstrap"; exit 127; }; swiftlint lint --strict --config apple/.swiftlint.yml; fi

lint-lines:
  max=500; \
  status=0; \
  while IFS= read -r -d '' file; do \
    lines="$(wc -l < "$file")"; \
    if [ "$lines" -gt "$max" ]; then \
      printf '%s has %s lines; max is %s\n' "$file" "$lines" "$max"; \
      status=1; \
    fi; \
  done < <(find . \
    -path './.git' -prune -o \
    -path './.qd/qd.db*' -prune -o \
    -path './.qd/worktrees' -prune -o \
    -path './target' -prune -o \
    -path './apple/*.xcodeproj' -prune -o \
    -path './apple/*.xcworkspace' -prune -o \
    -path './docs/PRD.md' -prune -o \
    -path './roadmap/qd-export.json' -prune -o \
    -name 'Cargo.lock' -prune -o \
    -name 'LICENSE-*' -prune -o \
    -type f \( \
      -name '*.rs' -o -name '*.swift' -o -name '*.toml' -o -name '*.md' -o \
      -name '*.yml' -o -name '*.yaml' -o -name '*.json' -o -name '*.nix' -o \
      -name 'justfile' -o -name '*.sh' \
    \) -print0); \
  exit "$status"

apple-generate:
  if [ "$(uname -s)" != Darwin ]; then echo "tuist generation skipped outside macOS"; else cd apple && if command -v tuist >/dev/null 2>&1; then tuist generate; elif command -v mise >/dev/null 2>&1; then mise exec -- tuist generate; else echo "tuist missing; run just macos-bootstrap"; exit 127; fi; fi

apple-test:
  if [ "$(uname -s)" != Darwin ]; then echo "apple tests skipped outside macOS"; else cd apple && { if command -v tuist >/dev/null 2>&1; then tuist generate; elif command -v mise >/dev/null 2>&1; then mise exec -- tuist generate; else echo "tuist missing; run just macos-bootstrap"; exit 127; fi; } && xcodebuild test -scheme MadobeMac -destination 'platform=macOS'; fi
