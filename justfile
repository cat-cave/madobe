set shell := ["bash", "--noprofile", "--norc", "-eu", "-o", "pipefail", "-c"]

default:
  just --list

fmt:
  cargo fmt --all -- --check
  if command -v taplo >/dev/null 2>&1; then taplo fmt --check; fi
  if command -v swiftformat >/dev/null 2>&1 && [ -d apple ]; then swiftformat --lint apple; fi
  if command -v shfmt >/dev/null 2>&1; then find . -path './.git' -prune -o -path './target' -prune -o -name '*.sh' -print0 | xargs -0 -r shfmt -d; fi

check: fmt
  cargo check --workspace --all-targets --all-features
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  if command -v typos >/dev/null 2>&1; then typos; fi
  if command -v markdownlint-cli2 >/dev/null 2>&1; then markdownlint-cli2 "**/*.md"; fi
  if command -v actionlint >/dev/null 2>&1; then actionlint .github/workflows/*.yml; fi
  if command -v swiftlint >/dev/null 2>&1 && [ -d apple ]; then swiftlint lint --strict --config apple/.swiftlint.yml; fi
  just lint-lines

test:
  if command -v cargo-nextest >/dev/null 2>&1; then cargo nextest run --workspace --all-features; else cargo test --workspace --all-features; fi
  cargo test --doc --workspace --all-features
  just apple-test

verify: check test security

coverage:
  command -v cargo-llvm-cov >/dev/null 2>&1 || { echo "cargo-llvm-cov missing; run inside nix develop"; exit 127; }
  cargo llvm-cov nextest --workspace --all-features --ignore-filename-regex 'apps/.*/src/main\.rs' --fail-under-lines 95 --lcov --output-path lcov.info

mutants:
  command -v cargo-mutants >/dev/null 2>&1 || { echo "cargo-mutants missing; run inside nix develop"; exit 127; }
  cargo mutants --workspace --timeout 120 --jobs 2

security:
  if command -v cargo-deny >/dev/null 2>&1; then cargo deny check; else echo "cargo-deny missing; run inside nix develop"; exit 127; fi
  if command -v cargo-vet >/dev/null 2>&1; then cargo vet; else echo "cargo-vet missing; run inside nix develop"; exit 127; fi
  if command -v cargo-audit >/dev/null 2>&1; then cargo audit; else echo "cargo-audit missing; run inside nix develop"; exit 127; fi
  if command -v cargo-machete >/dev/null 2>&1; then cargo machete --skip-target-dir; else echo "cargo-machete missing; run inside nix develop"; exit 127; fi
  if command -v cargo-semver-checks >/dev/null 2>&1; then cargo semver-checks check-release --workspace --baseline-root .; else echo "cargo-semver-checks missing; run inside nix develop"; exit 127; fi

ci-local: verify coverage

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
  if [ "$(uname -s)" = Darwin ] && command -v tuist >/dev/null 2>&1; then cd apple && tuist generate --no-open; else echo "tuist generation skipped outside macOS with tuist"; fi

apple-test:
  if [ "$(uname -s)" = Darwin ] && command -v tuist >/dev/null 2>&1; then cd apple && tuist generate --no-open && xcodebuild test -scheme MadobeMac -destination 'platform=macOS'; else echo "apple tests skipped outside macOS with tuist"; fi
