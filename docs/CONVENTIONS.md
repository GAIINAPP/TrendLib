# Numerical conventions

These rules apply to every indicator unless its `doc.md` states an exception, and
every exception must also appear in § 9.

## 1. Numbers

- Inputs and float outputs are IEEE-754 `float64`. Integer outputs are `int32`.
- The accuracy contract in `SPEC.md` § 6 covers **normal** `float64`. Subnormal
  inputs (magnitude below about 2.2e-308) are computed, raise nothing and
  produce a full-length output, but every operation on them drops significand
  bits, so two correct implementations can drift far past 1e-10 from each other
  and no tolerance against the oracle is claimed there.
- No fast-math, no reassociation flags. Do not use `mul_add` in one path (batch)
  and plain `a * b + c` in the other (stream): parity between TrendLib's own two
  paths is bitwise.
- Summation order matters for TA-Lib parity. Where TA-Lib uses a running sum
  (add newest, subtract oldest), do the same. Do not add Kahan or pairwise
  summation to a shared indicator unless parity with TA-Lib still holds within
  tolerance; if you do, document it.
- **Bitwise agreement with TA-Lib is not a goal and for some indicators is not
  reachable.** The contract against the oracle is the tolerance in `SPEC.md`
  § 6. Matching the arithmetic order gets SMA there exactly, because it has no
  multiply-add. It cannot get EMA there: ta-lib-python's published wheel
  evaluates `prev + (x - prev) * k` as one fused multiply-add, rounding once
  where a plain `a * b + c` rounds twice, so the two differ by up to one unit in
  the last place and then track each other (measured on 0.8.1, Linux x86-64,
  M1). Whether that contraction happens is a property of TA-Lib's build, not of
  the algorithm, so chasing it would make TrendLib's output depend on someone
  else's compiler flags. Each indicator's `doc.md` states which of the two it
  achieves, and the Python parity suite asserts it.

## 2. Alignment and warm-up

- Every output has the same length as the inputs.
- `lookback(params)` = the number of warm-up rows before the first defined value,
  for input with no leading NaNs. Example: SMA(period=5) has lookback 4; RSI(14)
  has lookback 14 (same values as TA-Lib's `*_Lookback`, which tests assert).
- Warm-up rows are `NaN` in float outputs and `0` in int32 outputs.
- Empty input returns empty outputs. Input no longer than the lookback returns all
  warm-up rows, with no error.

## 3. Missing and non-finite values

- **Leading NaNs are skipped.** The first valid bar is the first index where every
  input the indicator reads is finite. Rows before it are warm-up rows, and the
  lookback counts from the first valid bar: output row `k + lookback` is the first
  defined value when there are `k` leading invalid rows.
- **After the first valid bar, any NaN or ±inf in an input raises `InvalidInput`**
  naming the input and the row index (D8). Volume of `0` is valid; negative volume
  raises `InvalidInput`.
- Prices are not required to be positive (spreads and synthetic series can be
  negative). Indicators that need positivity (e.g. a log return) say so in
  `doc.md` and raise on non-positive input.

## 4. Seeding and smoothing

For every indicator TA-Lib also has, TrendLib reproduces TA-Lib's seeding and
smoothing exactly (D6). TA-Lib's behaviour is defined by running ta-lib-python
(the oracle), not by reading its source. In particular:

- EMA: smoothing factor `2 / (period + 1)`; the first value is the SMA of the first
  `period` values.
- Wilder smoothing (RSI, ATR, ADX family): first value is the simple average over
  the period, then `prev × (n − 1)/n + x/n`.
- There is no "unstable period" setting (D9). Outputs equal TA-Lib's with its
  default unstable period of 0. Users who want EMA-type values converged discard
  extra leading rows themselves; `doc.md` of affected indicators says so.

Do not copy TA-Lib source code. Re-implement from the formula and verify against
the oracle. If an obscure detail can only be matched by porting a TA-Lib routine,
keep TA-Lib's BSD-3-Clause notice in that file and add an entry to `NOTICE`.

## 5. Path dependence

Some indicators (Supertrend, VWAP, OBV, AD, ADX, anything recursive) depend on
where the computation starts. Their `spec.yaml` carries the `path_dependent` flag.

- Batch always starts at the first valid bar of the input it is given.
- A stream opened on history `H` and then fed bars `B` returns exactly the values
  batch returns for the series `H + B`.
- Batch on a slice can legitimately differ from the same rows of batch on the full
  series. `doc.md` says so for flagged indicators.

## 6. Streams

- `open(history)` requires at least `lookback + 1` valid bars after any leading
  NaNs; otherwise `InsufficientHistory` with the number needed.
- After `open`, `value` equals the last row of batch on `history`.
- `update(bar)` commits; `peek(bar)` returns what `update(bar)` would return and
  changes nothing; `copy()` is a deep, independent copy.
- A rejected bar (NaN, inf, negative volume, out-of-order timestamp) raises and
  leaves the stream exactly as it was.

## 7. Timestamps and sessions

- Accepted: pandas `DatetimeIndex`/Series with a timezone, NumPy `datetime64` with
  values interpreted as UTC, Python `datetime` with tzinfo, or int64 epoch
  nanoseconds (UTC). Naive pandas/Python datetimes raise `InvalidInput` asking
  the caller to localize (D10).
- Timestamps must be non-decreasing; equal timestamps are allowed.
- `tz` parameters take IANA names (`"Asia/Kolkata"`). The Python layer converts
  timestamps to epoch ns and passes per-bar UTC offsets to Rust, so the core crate
  needs no timezone database. (India has had no DST since 1945; the per-bar
  offsets still keep this correct for other zones.)
- `anchor="day"`: a new session starts when the local calendar date in `tz`
  changes. `anchor="none"`: never reset. More anchors (week, month, NSE trading
  calendar) are M6.

## 8. Output codes

- Trend direction outputs: `+1` up, `−1` down, `0` warm-up.
- Candlestick pattern outputs: `+100` bullish, `−100` bearish, `0` none or
  warm-up, matching TA-Lib. Candle settings (body/shadow thresholds) are fixed at
  TA-Lib's defaults in v0.1.

## 9. Deviations from TA-Lib

| # | Case | TA-Lib | TrendLib | Why |
| --- | --- | --- | --- | --- |
| 1 | NaN / inf after the first valid bar | Undefined, or skipped by some functions | Raises `InvalidInput` | D8 |
| 2 | VWAP rows before any volume has traded in the session | Carries previous value, `0` at the start | `NaN` | `0` is not a price |
| 3 | VWAP reset | Never resets; caller slices sessions | `anchor="day"` by default; alias `tl.VWAP` uses `anchor="none"` | Session VWAP is what Indian intraday users mean |
| 4 | Unstable-period setting | Global `TA_SetUnstablePeriod` | Not supported; equals TA-Lib default 0 | D9 |
| 5 | Candle settings | Globally configurable | Fixed at defaults (v0.1) | D9; configurable per call in M6 |
| 6 | `natr` with `period = 1` | Returns the raw true range, not normalised | Normalises at every period: `100 * atr / close` | A price span is not a percentage. TA-Lib's value is in price units under a name that means percent, which is the kind of plausible wrong number this project exists to avoid. Every other period agrees exactly. |

Golden tests against TA-Lib exclude only the rows a listed deviation affects, and
the golden header names the deviation number. Where a deviation makes the oracle
unusable at a parameter boundary rather than on particular rows, the boundary
case is generated at the nearest parameter the oracle can be trusted at and the
header carries a `# note:` saying which deviation forced it; a test refuses a
substitution that has no such note.

## 10. Known differences from TradingView

Recorded per indicator in `doc.md` under "Conventions", and only after checking
against TradingView output. The one confirmed at design time: TradingView's
`ta.supertrend` returns the opposite sign for direction (its −1 is an uptrend) and
seeds the initial trend differently; TrendLib follows TA-Lib (+1 = up).
