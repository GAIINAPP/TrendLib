---
name: new-indicator
description: Add one indicator to TrendLib end to end - spec.yaml, mod.rs, doc.md, oracle-produced golden CSVs, generated code and the full test suite. Use for every indicator from M4 onward, and whenever an existing indicator's spec, formula or golden data changes.
---

# Adding an indicator

One indicator is one folder plus the generated files it owns. Nothing else in
the repository should change, except `CHANGELOG.md`.

Read `docs/SPEC_FORMAT.md` and `docs/CONVENTIONS.md` before starting, and the
indicator's row in `docs/INDICATORS.md` section 2. Names, defaults and ranges
come from there or from an approved spec issue. If one is missing, stop and ask;
never invent one.

## 1. Check it is approved

The indicator must appear in `docs/INDICATORS.md` section 2 or have an approved
"New indicator spec" issue. Its oracle must be decided before you write code:
`T` (ta-lib-python, runnable) or `H` (human-transcribed reference values). No
oracle means no indicator.

## 2. Write the folder

`crates/trendlib/src/indicators/<name>/`:

- `spec.yaml` - metadata only. Copy the closest existing spec and edit it. Every
  name, default, range and output name is from section 1 above.
- `mod.rs` - the algorithm only. No defaults, no ranges, no display names.
  Implement `Indicator` and `Stream` sharing one step function, so bitwise
  parity holds by construction rather than by luck.
- `doc.md` - summary paragraph (under 60 words, descriptive, never advisory),
  `## Formula` with the original algebra, `## Conventions`, `## Example`,
  `## References`, in that order.

Rules that are easy to miss:

- `Stream::update` and `peek` never allocate; ring buffers are sized in `open`.
- `peek` must not touch state a later `update` reads.
- Errors are `TlError` values. A panic in the core crate is a bug.
- Warm-up rows are `NaN` for float outputs and `0` for int32 outputs, and the
  output length always equals the input length.
- If the result depends on where computation starts, set the `path_dependent`
  flag and say so in `doc.md`.

## 3. Produce golden data by running the oracle

```bash
cargo xtask golden <name>              # T: runs scripts/oracle/talib_golden.py
```

`golden/default.csv` is required; add a case for each boundary parameter, at
minimum the minimum period. The header keys in `docs/SPEC_FORMAT.md` section 4
are all required, including `excluded_rows` and the tolerance.

For an `H` oracle, transcribe the values unchanged, record the platform, symbol,
transcriber and date in the `oracle` header, and set `produced_by: manual`.
Until a human supplies them, mark the test `#[ignore = "awaiting oracle (Q4)"]`
and list it in the milestone report.

**Never** compute expected values with TrendLib's own formula or a
re-derivation of it. That is not an oracle: it agrees for exactly the reason it
must not.

## 4. Generate, then test

```bash
cargo xtask generate
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
maturin develop --release && pytest -q
cargo xtask regen-check
```

Never hand-edit a file carrying the `@generated` banner. If the output is wrong,
the spec or the generator is wrong.

## 5. Prove the golden test can fail

Break the implementation once - an off-by-one in the window start is the usual
choice - run the golden test, confirm it goes red, restore the code, and state
in the PR exactly how you broke it. A green test that never ran against broken
code proves nothing.

## 6. Finish

- Add a line to `CHANGELOG.md` under "Unreleased".
- Check the generated docs page reads correctly.
- Confirm no function, output, parameter or doc sentence reads as a trade
  instruction (`docs/DECISIONS.md` D11). MACD's signal line is fine; "entry" is
  not.
- The diff touches the indicator folder, the generated files and the changelog.
  Anything else needs a reason in the PR.
