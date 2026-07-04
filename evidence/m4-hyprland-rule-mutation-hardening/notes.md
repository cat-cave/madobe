# m4-hyprland-rule-mutation-hardening Notes

## Baseline Cluster

The qd node supplied the relevant workspace mutation baseline cluster from
main:

- `crates/hyprland/src/adapter/rule.rs`: `f64_to_nonzero_u32` survived
  boolean/operator boundary mutations.
- `crates/hyprland/src/adapter/rule.rs`: `format_scale` survived a
  division/operator mutation.
- `crates/hyprland/src/adapter/rule.rs`: `lua_string` survived an
  escape-condition mutation.
- `crates/hyprland/src/adapter/rule.rs`: `greatest_common_divisor` survived
  comparison/modulo mutations.

## Coverage Added

Added focused unit tests in `crates/hyprland/src/adapter/rule.rs`:

- `f64_to_nonzero_u32_rejects_non_positive_and_out_of_range_values` covers
  non-finite, negative, zero, below-first-rounded-nonzero, and
  above-`u32::MAX` rejection.
- `f64_to_nonzero_u32_accepts_rounded_u32_boundaries` covers rounding behavior
  from `0.5` and the exact `u32::MAX` upper boundary.
- `format_scale_formats_integer_and_fractional_scales` covers integer scales
  with denominator `1` and non-`1`, trimmed decimals, and three-decimal
  rounding.
- `monitor_rule_formats_scale_via_config` exercises the public
  `monitor_rule` path for fractional scale formatting.
- `lua_string_escapes_quotes_and_backslashes` covers both escaped character
  branches.
- `config_from_monitor_rounds_and_reduces_scale_thousandths` parses monitor
  JSON and verifies refresh-rate rounding plus scale reduction from
  thousandths through `config_from_monitor`.
- `greatest_common_divisor_handles_zero_and_nontrivial_remainders` pins the
  Euclidean helper for zero, reducible, coprime, and multi-step remainder
  cases.

## Mutation Check

Initial targeted run:

```text
nix develop -c cargo mutants -p madobe-hyprland --file crates/hyprland/src/adapter/rule.rs --timeout 120 --jobs 2 --all-features --output /tmp/madobe-rule-mutants-m4-hyprland-rule-mutation-hardening
43 mutants tested in 15s: 1 missed, 24 caught, 18 unviable
```

The survivor was the integer `format_scale` branch replacing `/` with `*`.
The original integer assertion used a denominator of `1`, so both operators
returned the same value. I added `format_scale(scale(4, 2)) == "2"`.

Final targeted run:

```text
nix develop -c cargo mutants -p madobe-hyprland --file crates/hyprland/src/adapter/rule.rs --timeout 120 --jobs 2 --all-features --output /tmp/madobe-rule-mutants-m4-hyprland-rule-mutation-hardening-nonzero-final
45 mutants tested in 16s: 27 caught, 18 unviable
```

There were no missed rule.rs mutants and no timeouts in the final targeted
run. Cargo-mutants output was written under `/tmp` to avoid committing large
generated JSON artifacts that repository line-count linting would scan.

## Scope

This node only hardens Hyprland monitor rule/config helper tests. It does not
change transport, product QUIC, capture, encode, Apple, Nix, CI,
supply-chain, qd topology, or unrelated docs.
