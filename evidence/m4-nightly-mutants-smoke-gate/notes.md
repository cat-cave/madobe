# m4-nightly-mutants-smoke-gate Notes

## Root Cause

Scheduled nightly run `28737770881` failed in
`nix develop -c just mutants` because the recipe was running the full workspace
`cargo mutants --workspace --timeout 120 --jobs 2` sweep. The supplied job log
summary was:

```text
759 mutants tested in 5m: 67 missed, 349 caught, 342 unviable, 1 timeouts
```

That full workspace run is currently useful as a baseline discovery tool, but
it is not a passing nightly gate.

## Smoke Gate Scope

`just mutants` now runs the curated targeted checks that prior node evidence
showed were already clean on Linux/Nix:

- `madobe-compositor`, `crates/compositor/src/ids.rs`
- `madobe-compositor`, `crates/compositor/src/status.rs`
- `madobe-encode-nv-sys`, `crates/encode-nv-sys/src/lib.rs`

The recipe writes cargo-mutants output to a `mktemp -d` directory and removes it
with a shell trap so the working tree stays clean.

`just mutants-full` preserves the previous exploratory full workspace command:

```text
cargo mutants --workspace --timeout 120 --jobs 2
```

I did not run the full workspace sweep for this node because the acceptance
criteria only require preserving and documenting the command.
