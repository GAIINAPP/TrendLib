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
- TA-Lib parity with `hypothesis`: random OHLCV and params, all shared indicators,
  tolerance from § 4; `tl.lookback(...)` equals `talib.abstract.Function(...).lookback`.
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
