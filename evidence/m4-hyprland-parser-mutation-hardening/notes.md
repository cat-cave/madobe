# m4 Hyprland Parser Mutation Hardening Notes

## Baseline

The qd node supplied the full workspace cargo-mutants baseline from main:

```text
751 mutants tested in 8m: 105 missed, 302 caught, 340 unviable, 4 timeouts
```

The parser cluster called out by the node covered missed mutants around
control-character rejection in strings, Unicode escape decoding, JSON number
grammar boundaries, `is_finished` behavior, and timeout-prone cursor progress
in `skip_whitespace`, `consume_digits`, and `next_byte`.

## Changes

Added focused parser unit tests in `crates/hyprland/src/model/parser.rs` for:

- raw string control characters at `0x00`, `0x1f`, and newline;
- regular string escapes and `\uNNNN` Unicode escape decoding;
- invalid truncated and non-hex Unicode escapes;
- valid JSON number sign, fraction, and exponent forms;
- invalid number forms such as `-`, leading zeroes, `1.`, and incomplete
  exponents;
- root parser trailing non-whitespace rejection; and
- cursor progress through whitespace, digits, `next_byte`, and `is_finished`.

The parser scanner was tightened where the new tests exposed real leniency:
numbers now require a digit after an optional sign, reject leading zeroes, and
require digits after a decimal point or exponent marker.

The timeout-prone cursor loops were also reshaped. `skip_whitespace` and
`consume_digits` now count matching bytes from the remaining slice instead of
looping around manual cursor increments. `next_byte` advances with
`saturating_add(1)`.

## Mutation Evidence

First targeted parser run:

```text
79 mutants tested in 2m: 35 caught, 43 unviable, 1 timeouts
```

The remaining timeout was `Parser::next_byte` with cursor progress mutated
from `+=` to `*=`.

Final targeted parser run after the `next_byte` change:

```text
78 mutants tested in 25s: 35 caught, 43 unviable
```

There were no missed parser mutants and no parser timeouts in the final
targeted run.

The raw cargo-mutants output directories were removed because this repository's
`lint-lines` recipe scans JSON under `evidence/`, and the generated
`mutants.json` and `outcomes.json` files exceeded the 500-line cap. The
command summaries above are preserved in `commands.log`.

## Residual Risk

The post-change mutation check was scoped to `madobe-hyprland` and
`crates/hyprland/src/model/parser.rs`. The full workspace mutation suite was
not rerun after this focused parser hardening.
