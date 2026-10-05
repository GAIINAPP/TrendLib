# Testing

A library of numbers is only as good as the evidence that the numbers are right.
TrendLib's evidence is an **independent, executed oracle** for every indicator,
plus suites that prove batch, stream and Python surfaces agree.

## 1. Test layers

| Layer | Where | What it proves | Runs |
| --- | --- | --- | --- |
| Golden | `crates/trendlib/tests/golden.rs` | Batch output matches the oracle within tolerance | Every PR |
| Stream parity (generated) | `crates/trendlib/tests/stream_parity.rs` | Stream equals batch bitwise; `peek` commits nothing; `copy` is independent | Every PR |
| Edge cases (generated) | `crates/trendlib/tests/edge_cases.rs` | Defined behaviour on hostile input (§ 5) | Every PR |
| Python contract | `tests/test_api_*.py` | `PYTHON_API.md`: types in/out, index, names, errors | Every PR |
| TA-Lib parity (property) | `tests/test_talib_parity.py` | Random data and params agree with TA-Lib; lookbacks equal | Every PR (bounded examples), nightly (more) |
| Pattern coverage | `tests/test_patterns.py` | Every pattern has a golden it fires in, and agrees with TA-Lib on random bars | Every PR |
| Chart pattern coverage | `tests/test_chart_patterns.py` | Every chart pattern has a golden it fires in in both directions it reads, and agrees with `ta-patterns` on random bars | Every PR |
| Wheel smoke | `release.yml` | Installed wheel imports and computes on each platform | Every release, TestPyPI and PyPI |
| Benchmarks | `benches/`, `tests/bench/` | Speed vs TA-Lib | Nightly; enforced by release checklist |

## 2. Oracles

The expected values in a golden file must be **produced by running** an
independent implementation, or transcribed unchanged from a published source.
Re-deriving the formula yourself and labelling the result with a vendor's name is
forbidden: it agrees with TrendLib for exactly the reason it must not.

**T — ta-lib-python 0.8.1.** Pinned in the `dev` extra as `TA-Lib==0.8.1` (it ships
wheels; no C install needed). `scripts/oracle/talib_golden.py <name> --case <case>`
loads the dataset, calls the TA-Lib function with the case's parameters (renamed per
`spec.yaml` `talib:`), writes the CSV with the header from `SPEC_FORMAT.md` § 4, and
marks rows affected by a `CONVENTIONS.md` § 9 deviation as excluded. For `vwap` it
runs TA-Lib `VWAP` once per session slice.

**P — ta-patterns 1.2.1.** For the chart patterns of `INDICATORS.md` section 5.1,
which TA-Lib does not have. Pinned in the `dev` extra as `ta-patterns==1.2.1`
(MIT, pure NumPy, no compiled code). `scripts/oracle/chart_golden.py <name>
--case <case>` calls the oracle function the table in section 5.1 names, with the
parameters renamed back (`period` is its `window`), multiplies its `+1`/`-1` by
100, and writes the file in the format of `SPEC_FORMAT.md` § 4. For the two-sided
functions built from two oracle functions (`chart_broadening`, `chart_rectangle`)
the upward reading is taken where it fired and the downward one elsewhere, which
is the order TrendLib tests them in. Rows a `CONVENTIONS.md` § 9 deviation (8 or
9) touches are excluded and named in the header; the script finds them with the
oracle's own swing-point functions, not with TrendLib.

**H — human-transcribed.** For `cpr`, `pivots_traditional`, `pivots_camarilla` and
future indicators with no runnable reference. Accepted sources, in order of
preference:

1. Values read off TradingView ("Pivot Points Standard", type Traditional or
   Camarilla; a CPR script named in the header) for listed dates of a listed symbol.
   Up to 30 rows of real daily OHLC may be committed for this purpose only (a
   documented exception to D13).
2. A published calculator or worked example run on synthetic inputs, named in the
   header.

**A — the approved formula evaluated through an independent implementation's
arithmetic.** For the three `levels` functions, which no library has an
equivalent of. The formulas `INDICATORS.md` § 3.2 to 3.4 approve are evaluated
through TA-Lib's own `TYPPRICE`, `MEDPRICE`, `ADD`, `SUB`, `MULT` and `DIV`, so
every number in the file comes out of the C library and only the shape of the
expression is ours. It is weaker than T, because an error in the approved
formula would go unnoticed, and stronger than values TrendLib computed for
itself, which the rule above forbids. The header of each such file says which
functions produced it.

H is still the stronger evidence for those three and open question Q4 still
asks for it; A is what ships until then. Do not substitute self-computed
values for either.

**Upgrading an oracle version** regenerates every golden from that oracle in one
PR whose description summarises any value changes.

## 3. Test data

`scripts/testdata/make_synthetic.py` writes, with a fixed seed:

| File | Shape | Notes |
| --- | --- | --- |
| `testdata/daily_2000.csv` | 2,000 daily OHLCV bars | Geometric random walk from 1000; `low ≤ min(open, close)`, `high ≥ max(open, close)`; integer volumes |
| `testdata/intraday_5m_20d.csv` | 20 sessions × 75 bars, 09:15–15:25 IST starts, `timestamp` = epoch ns UTC | Session 3 opens with a zero-volume bar; one mid-session zero-volume bar; one session spans a weekend gap |
| `testdata/patterns_1080.csv` | 1,080 daily OHLCV bars | 31 hand-built shapes, one per pattern a walk does not reach, each after 12 quiet bars; then a 600-bar walk through ten regimes |
| `testdata/charts_2400.csv` | 2,400 daily OHLCV bars | Hand-built chart shapes, each direction of each section 5.1 pattern at least once, separated by drifting filler; then a walk through trending and ranging regimes |

A pattern fires on a handful of bars or on none at all. Over `daily_2000.csv` 31 of
the 61 patterns fire fewer than five times and 12 never fire, so their golden files
hold nothing but zeros and assert only that the pattern stayed silent — a rule can be
wrong in ways such a file cannot see, and nine were. `patterns_1080.csv` exists so
every pattern has a golden with at least one firing bar in it; `tests/test_patterns.py`
fails if one of them goes quiet again.

The chart patterns have the same blind spot, wider: over `daily_2000.csv` the
triangles, rectangles and broadening formation never fire at all, because a
flat line is judged by its slope in price per bar and a random walk near 1,000
almost never draws one. `charts_2400.csv` is built the way `patterns_1080.csv`
was, for them, and each chart pattern's `charts` golden reads it;
`tests/test_chart_patterns.py` fails if one of them goes quiet in either
direction it can read.

The shapes are chosen so the oracle reports the pattern. They decide *which bars* a
golden file covers, never what the expected values are: those still come from running
TA-Lib (section 2), the same as every other golden.

The committed CSVs are the source of truth. NumPy does not promise identical random
streams across versions, so regenerate only on purpose and regenerate goldens in the
same PR.

## 4. Golden comparison

- Float outputs: pass if `|actual − expected| ≤ abs` or `≤ rel × |expected|` (from the
  header). NaN must be NaN in exactly the same rows.
- Int outputs: exact equality.
- Excluded rows (header) are skipped and counted in the test output.
- The runner also opens a stream on the first `lookback + 1` valid rows, feeds the
  rest, and requires the values to equal the batch output bitwise.

**Prove each golden can fail.** When adding an indicator, break the implementation
(e.g. off-by-one in the window), confirm the golden test goes red, restore it, and
state this in the PR. A green test that never ran against broken code proves nothing.

## 5. Generated edge-case suite

For every indicator, with default params unless stated:

| Case | Expected |
| --- | --- |
| Empty input | Empty outputs, no error |
| Length < lookback, = lookback, = lookback + 1 | All warm-up; all warm-up; exactly one defined row |
| Constant series | No error; values documented (e.g. RSI of a flat series per TA-Lib) — compared to TA-Lib where available |
| `k` leading NaNs | Output equals batch on the trimmed input, shifted by `k`, with warm-up prefix |
| NaN or ±inf after the first valid bar | `InvalidInput` naming input and row |
| Negative volume (volume indicators) | `InvalidInput` |
| Decreasing timestamps (timestamped indicators) | `InvalidInput` |
| Magnitudes 1e-300 and 1e+300 | No panic; any non-finite output is documented in `doc.md` |
| Each param at `min` and `max` | No error (all warm-up allowed) |
| Each param at `min − 1` / `max + 1`, wrong enum value | `InvalidInput` with the range in the message |
| Stream opened with `lookback` bars | `InsufficientHistory` |
| Stream fed NaN | `InvalidInput`; stream unchanged (next valid bar gives the same value as without the bad call) |

## 6. Stream parity (generated, property-based)

With `proptest`, for each indicator: random valid series (length 0–600), random
params within range (periods capped at 60 for speed), random split point `s ≥
lookback + 1`:

- `batch(series)` equals `open(series[..s])` followed by `update` for each later bar,
  bitwise (NaN == NaN).
- `peek(bar)` equals the following `update(bar)`; calling `peek` twice changes nothing.
- `copy()` then updating the copy leaves the original's next values unchanged.
- `open_and_fill` returns the same as `batch` on the history.

## 7. Python tests

- Contract tests for every rule in `PYTHON_API.md` (types, index, column names,
  DataFrame mapping, errors, aliases, accessor, registry, `to_json` schema).
- TA-Lib parity over the committed datasets: every shared indicator, every enum
  value, swept parameters, a constant series and a leading run of NaN, tolerance
  from § 4; `tl.lookback(...)` equals `talib.abstract.Function(...).lookback`.
- TA-Lib parity with `hypothesis`: random series against the eight indicators whose
  arithmetic is worth fuzzing (`sma`, `ema`, `wma`, `rsi`, `atr`, `macd`, `cci`,
  `willr`). It does not cover the patterns; `tests/test_patterns.py` does, over 24
  random series of 750 bars built to contain gaps, doji bars and marubozu runs.
- Alias equivalence: `tl.RSI(x, timeperiod=n)` equals `tl.rsi(x, period=n)`.
- polars tests skip when polars is missing; one CI job installs it so they always run
  somewhere.

## 8. Benchmarks

- Rust: criterion, 1,000,000 bars per indicator, default params.
- Python: `pytest-benchmark`, `tl.<name>` vs `talib.<NAME>` on the same 1,000,000-bar
  NumPy arrays.
- Nightly job posts a table to the job summary. The release checklist
  (`RELEASE.md`) blocks a release if any shared indicator's median is more than 1.5×
  TA-Lib's.

## 9. CI matrix (`ci.yml`)

| Job | OS | Python | Does |
| --- | --- | --- | --- |
| lint | ubuntu | — | `cargo fmt --check`, `clippy -D warnings`, `ruff check`, `ruff format --check`, `cargo deny check` |
| rust | ubuntu, macOS, windows | — | `cargo test --workspace` |
| python | ubuntu, macOS, windows | 3.11, 3.14 | `maturin develop --release`, `pytest` (with pandas; polars on ubuntu 3.14 only) |
| regen | ubuntu | 3.14 | `cargo xtask regen-check` (from M2) |
| api | ubuntu | 3.14 | public API snapshot unchanged, or the PR is labelled `api-change` |
