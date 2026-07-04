# m4 Hyprland Model Accessor Mutation Hardening Notes

## Baseline Cluster

The qd node supplied the relevant workspace mutation baseline cluster from
main for `crates/hyprland/src/model.rs`:

- monitor `description` survived replacement mutants;
- monitor `scale` survived replacement mutants;
- monitor `dpms_status` survived replacement mutants;
- monitor `disabled` survived replacement mutants;
- workspace `monitor_id` survived replacement mutants;
- workspace `has_fullscreen` survived replacement mutants; and
- workspace `is_persistent` survived replacement mutants.

The broader workspace mutation baseline from the adjacent parser-hardening node
was:

```text
751 mutants tested in 8m: 105 missed, 302 caught, 340 unviable, 4 timeouts
```

## Changes

Added focused unit tests in `crates/hyprland/src/model.rs` that parse inline
Hyprland monitor and workspace JSON fixtures, then assert the accessor behavior
through the public parsed model API.

The monitor fixture uses two parsed monitors with distinct non-empty
descriptions, non-default scales, and opposite `dpmsStatus` and `disabled`
values:

- `description()` returns `Madobe virtual display A` and
  `Madobe virtual display B`;
- `scale()` returns `1.25` and `0.75`;
- `dpms_status()` returns both `false` and `true`; and
- `disabled()` returns both `true` and `false`.

The workspace fixture uses two parsed workspaces with monitor IDs that differ
from workspace IDs and opposite fullscreen and persistent values:

- `monitor_id()` returns `31` and `32`;
- `has_fullscreen()` returns both `true` and `false`; and
- `is_persistent()` returns both `true` and `false`.

## Mutation Evidence

Final targeted mutation run:

```text
nix develop -c cargo mutants -p madobe-hyprland \
  --file crates/hyprland/src/model.rs \
  --timeout 120 --jobs 2 --all-features \
  --output /tmp/madobe-hyprland-model-mutants-final

70 mutants tested in 31s: 58 caught, 12 unviable
```

There were no missed mutants and no timeouts in the targeted `model.rs` run.
The cargo-mutants output directory was kept under `/tmp` instead of
`evidence/` to avoid committing generated JSON that would be scanned by
repository line-count linting.

## Validation

Final targeted package test:

```text
nix develop -c cargo test -p madobe-hyprland --all-features
```

Result: passed.

Final repository check:

```text
nix develop -c just check
```

Result: passed.

## Residual Risk

The mutation check was scoped to `madobe-hyprland` and
`crates/hyprland/src/model.rs`, matching this node. The full workspace mutation
suite was not rerun after this focused accessor hardening.
