# madobe Engineering System

Status: engineering proposal, July 2, 2026

This document defines how the madobe monorepo should be built, checked, tested, reviewed, and validated. The goal is high velocity without quality debt. The reference stack is narrow and modern, so the engineering system should optimize for proving that stack deeply rather than pretending to support broad compatibility.

## Operating Principles

- Reference stack first: CI must prove Linux 7.x+ Hyprland/NVIDIA hosts and M5 Pro macOS clients, not generic Linux or old Apple hardware.
- Strict by default: formatting, linting, typechecking, tests, dependency policy, and docs are merge blockers.
- No silent escape hatches: diagnostic paths are allowed for investigation, but they do not count as product validation.
- Unsafe is exceptional: `unsafe` is isolated to FFI/interop crates with safety proofs, extra tests, and named owners.
- Hardware truth beats mocks: unit tests prove logic, simulators prove state machines, hardware CI proves the product.
- Ratchets only move upward: coverage, mutation score, lint strictness, and performance budgets should get tighter over time.

## Monorepo Shape

```text
apps/
  hostd/              Rust user-session daemon
  cli/                madobectl diagnostics and manual control
  macos/              macOS app shell, Xcode/Tuist/XcodeGen project

crates/
  protocol/           wire schema, negotiation, frame headers
  transport/          QUIC session, lane scheduling, reconnect
  hyprland/           IPC, output lifecycle, workspace reconciliation
  capture/            PipeWire/XDPH and direct capture adapters
  encode-nv/          safe NVENC product API
  encode-nv-sys/      raw NVIDIA SDK FFI, unsafe allowed only here
  graphics-interop/   DMA-BUF, Vulkan/CUDA/EGL interop wrappers
  audio/              PipeWire audio and Opus
  input-linux/        uinput, evdev, EIS experiments
  telemetry/          traces, counters, debug bundles
  testkit/            fakes, simulators, golden fixtures

apple/
  Packages/
    ProtocolCore/
    MadobeClientCore/
  MadobeMac/

nix/
  flake.nix
  modules/
  checks/
  overlays/

ci/
  scripts/
  hardware/
  perf/

xtask/
  src/main.rs         repo automation
```

Keep shared protocol and state-machine logic in Rust unless there is a strong reason not to. The macOS app can bind to Rust through FFI for protocol/session logic or use generated Swift bindings, but render/decode/audio/input UI code stays native.

## Toolchain

Use `just` as the shared command surface. Toolchain boundaries are platform-specific.

Linux uses Nix as the authoritative toolchain boundary:

- Pin Rust stable and nightly where needed.
- Pin clang/LLVM, bindgen, pkg-config, PipeWire headers, Vulkan SDK pieces, CUDA/NVIDIA SDK glue, GStreamer/FFmpeg tools, and cargo tools.
- Expose one `nix develop` shell with every required tool.
- Linux CI must use the same flake inputs as local development.

macOS uses native Xcode plus Mise/Homebrew:

- Pin Tuist and Rust tools in `.mise.toml`.
- Install SwiftFormat and SwiftLint through the native macOS environment.
- Do not require Nix on macOS CI or MacBook orchestrator sessions.

Human-facing commands should live behind `cargo xtask` or `just`. The command should not assume which platform
manager put the tools on PATH.

Recommended commands:

```text
just fmt
just check
just test
just verify
just coverage
just mutants
just hardware
```

## Rust Standards

Every safe Rust crate should start with:

```rust
#![forbid(unsafe_code)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
#![deny(unreachable_pub)]
#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
```

Do not enable all of `clippy::restriction`; choose restriction lints deliberately. Baseline restriction lints:

- `clippy::unwrap_used`
- `clippy::expect_used`
- `clippy::panic`
- `clippy::todo`
- `clippy::unimplemented`
- `clippy::dbg_macro`
- `clippy::print_stdout`
- `clippy::print_stderr`
- `clippy::allow_attributes_without_reason`

FFI and interop crates are the only crates allowed to use `unsafe`:

- Unsafe crates must use `#![deny(unsafe_op_in_unsafe_fn)]`.
- Every unsafe block requires a precise `// SAFETY:` explanation.
- Unsafe APIs must be wrapped by narrow safe abstractions.
- Unsafe crates require extra owner review.
- Unsafe crates must have Miri/sanitizer/fuzz coverage where applicable.

## Formatting and Style

Repository-wide:

- `.editorconfig` for indentation, newline, charset, and final newline.
- Rustfmt with a fixed max width.
- SwiftFormat with a fixed max width.
- Taplo for TOML.
- Prettier or markdownlint for Markdown/YAML if introduced.
- ShellCheck and shfmt for shell scripts.
- `typos` for spelling.

Generated files must be clearly marked and excluded only when regeneration is deterministic and checked elsewhere.

Linux `nix develop -c just check` is the required repository hygiene gate for
Rust formatting, TOML formatting, shell formatting, spelling, Markdown linting,
GitHub Actions linting, and line-limit checks. These Linux hygiene tools are
required in `PATH`; missing tools must fail fast instead of skipping checks.
On macOS, the same hygiene checks run when the tools are installed and otherwise
print explicit non-Linux skips so native `just macos-check` remains focused on
Rust, SwiftFormat, SwiftLint, Tuist generation, and Xcode tests.

## CI Gates

### PR Fast Gate

Runs on every PR:

```text
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
cargo nextest run --workspace --all-features
cargo test --doc
cargo deny check
cargo vet
cargo semver-checks
swiftformat --lint
swiftlint lint --strict
xcodebuild build
xcodebuild test
typos
taplo fmt --check
shellcheck
```

Use `cargo-nextest` as the Rust test runner for speed, per-test isolation, sharding, retries where explicitly allowed, JUnit output, and CI artifacts.

### Merge Gate

Required before merge:

- Fast gate passes.
- Coverage thresholds pass.
- No new unsafe outside approved crates.
- No new dependency without `cargo-deny` and `cargo-vet` status.
- No public API change without docs and semver check.
- No protocol/capture/encode/input/security change without an ADR.
- Hardware smoke passes when the change affects product-critical paths.

### Nightly Gate

Runs on schedule:

- Full workspace coverage.
- Mutation testing on selected crates.
- Fuzz corpus runs.
- Miri on eligible crates.
- Sanitizer runs on FFI/interop crates.
- Longer QUIC loss/reconnect simulations.
- Hardware soak on reference hosts.

## Coverage and Mutation Testing

Use `cargo-llvm-cov` with `cargo-nextest`.

Coverage targets:

- `protocol`: 95%+ line coverage, high branch coverage, mutation-gated.
- `transport`: 90%+ line coverage plus loss/reorder property tests.
- `hyprland`: 90%+ line coverage with JSON/event fixtures.
- `telemetry`: 95%+ line coverage and golden trace tests.
- `permissions`: near-total branch coverage.
- FFI/interop crates: lower line coverage is acceptable only with integration, sanitizer, and hardware tests.

Use `cargo-mutants` as a ratchet:

- Required for `protocol`, permission policy, telemetry math, frame scheduling, reconnect/session state.
- Nightly full run.
- PR targeted run for touched high-risk crates once runtime is acceptable.
- Surviving mutants must become tests, documented equivalences, or explicit design decisions.

## Fuzzing and Property Tests

Use property tests for:

- protocol encode/decode round trips
- frame fragmentation/reassembly
- sequence number wrap behavior
- reconnect state machines
- permission policy transitions
- file transfer accounting
- color metadata conversions
- telemetry timestamp ordering

Use fuzzing for:

- protocol parsers
- clipboard payload parsing
- file transfer manifests
- debug bundle import/export
- QUIC lane frame parsing
- any FFI-facing external data

Crashes become regression fixtures.

## Hardware Validation

Hosted CI proves code hygiene. Hardware CI proves madobe.

Linux hardware runner:

- Linux 7.x+.
- Hyprland.
- NVIDIA RTX 4090 or RTX 5060 Ti-class GPU.
- Pinned NVIDIA driver and CUDA/NVENC SDK stack.
- PipeWire and XDPH.
- Isolated test user and runtime dir.
- Ability to create headless outputs without touching the developer session.

macOS hardware runner:

- M5 Pro.
- Pinned Xcode.
- VideoToolbox AV1 decode.
- Metal render validation.
- CoreAudio output tests.

Required hardware artifacts:

- host and client logs
- frame timeline JSON
- Perfetto trace
- encoded AV1 sample
- decode/render timing
- capture metadata: DRM format, modifier, device, sync mode
- QUIC stats
- redacted debug bundle

Hardware jobs should be split:

- smoke: short, merge-blocking for critical paths
- performance: scheduled or manual, merge-blocking only for performance-sensitive changes
- soak: overnight, scheduled

## Performance Budgets

Performance regressions are correctness failures for this project.

Track:

- capture start to DMA-BUF available
- DMA-BUF import/convert time
- encode submit to encoded output
- QUIC enqueue to client receive
- client receive to decode output
- decode output to Metal present
- input sample to host injection
- input sample to visible response when measurable
- dropped-before-encode, dropped-before-send, dropped-after-receive, decoder-late, renderer-late
- CPU readback count, which must be zero on the product path

Initial product budgets should be aggressive and explicit once the first measurements exist. Until then, every trace must expose p50, p95, p99, max, queue depth, and frame interval misses.

## Swift/macOS Standards

Use SwiftFormat and SwiftLint in CI and locally.

Client rules:

- No force unwraps except test fixtures or explicitly justified boundary code.
- No blocking work on decode, render, audio, or input paths.
- No hidden global mutable state.
- Prefer structured concurrency, but do not put realtime paths behind actor hops without measurement.
- All permission prompts and OS-owned shortcut behavior must be tested or documented.
- Xcode result bundles should be uploaded for CI failures.

Keep generated Xcode project churn out of reviews. Prefer:

- Swift packages for client logic.
- Tuist or XcodeGen for reproducible projects, or a very thin checked-in Xcode project.
- Package-resolved files committed and reviewed.

## Dependency and Supply Chain

Rust:

- `cargo-deny` blocks disallowed licenses, advisories, duplicate crates, yanked crates, and unexpected sources.
- `cargo-vet` records dependency audits and imported trusted audits.
- Git dependencies are banned unless tied to an upstream/fork decision record.
- FFI dependencies require license, redistribution, and packaging review.

Swift:

- Pin Swift packages.
- Avoid package plugins in unattended CI unless explicitly trusted.
- Review binary artifacts and generated code.

Nix:

- Every non-trivial external tool enters through the flake.
- Overrides and forks need owner, reason, and exit criteria.

macOS:

- Project generation enters through Tuist.
- Mise-pinned tools should be updated deliberately and reviewed like other dependency changes.
- Homebrew-installed CI tools must be named explicitly in the workflow.

## ADRs and Review

Architecture decision records are required for:

- protocol schema changes
- codec default changes
- capture backend changes
- input injection model changes
- permission/security model changes
- unsafe/FFI expansion
- dependency forks
- hardware CI policy changes

Use CODEOWNERS for:

- protocol
- transport
- capture
- encode/FFI
- input
- permissions/security
- macOS decode/render
- Nix/CI

No PR self-merges for owned areas.

## Release Discipline

A release is not a tag; it is evidence.

A release candidate requires:

- all merge gates green
- hardware smoke green on both host shapes
- macOS hardware smoke green
- no product-path CPU readback
- AV1 path validated
- permission model tests green
- debug bundle generated and redacted
- performance trace attached
- known failures documented

No release should claim support for anything outside the reference stack unless a later decision record deliberately expands scope.

## Source Notes

- `cargo-nextest`: fast Rust test runner with CI support, partitioning, JUnit, recordings, and traces: <https://nexte.st/>
- `cargo-mutants`: Rust mutation testing: <https://mutants.rs/>
- `cargo-vet`: dependency audit workflow: <https://mozilla.github.io/cargo-vet/>
- `cargo-deny`: dependency graph policy checks: <https://embarkstudios.github.io/cargo-deny/>
- Clippy lint categories and restriction guidance: <https://doc.rust-lang.org/clippy/index.html>
- `cargo-llvm-cov`: LLVM coverage with nextest support and thresholds: <https://github.com/taiki-e/cargo-llvm-cov>
- SwiftLint: <https://github.com/realm/SwiftLint>
- SwiftFormat: <https://github.com/nicklockwood/SwiftFormat>
