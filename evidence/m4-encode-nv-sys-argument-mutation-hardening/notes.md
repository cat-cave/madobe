# m4 Encode NV Sys Argument Mutation Hardening Notes

## Baseline

The qd node supplied the relevant workspace mutation baseline cluster from
main for `crates/encode-nv-sys/src/lib.rs`:

- `NvencPreset::ffmpeg_name` survived replacement with empty and `xyzzy`
  strings.
- `NvencTune::ffmpeg_name` survived replacement with empty and `xyzzy`
  strings.
- `shell_arg` survived a boolean operator mutation from `||` to `&&`.

## Changes

Added focused unit tests in `crates/encode-nv-sys/src/lib.rs` for:

- exact `NvencPreset::P4.ffmpeg_name()` output of `p4`;
- exact `NvencTune::Ll.ffmpeg_name()` output of `ll`;
- `shell_arg` preserving safe ffmpeg tokens containing alphanumeric
  characters and the allowed `/`, `.`, `_`, `-`, `:`, and `=` characters; and
- `shell_arg` quoting empty strings, whitespace-containing values, and values
  with single quotes.

The empty string case now returns `''` instead of an empty display fragment, so
the evidence command renderer preserves explicit empty arguments if one ever
reaches this boundary.

## Validation

Required validation passed:

```text
nix develop -c cargo test -p madobe-encode-nv-sys --all-features
```

Result: 5 tests passed.

```text
nix develop -c just check
```

Result: passed.

## Residual Risk

Validation was focused on unit coverage and the repository check required by
the node. I did not rerun the full workspace mutation suite.
