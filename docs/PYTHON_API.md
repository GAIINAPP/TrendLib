# Python API contract

This is the public surface. Anything here is covered by SemVer from 1.0; before
1.0 changes are allowed but must be listed in `CHANGELOG.md`.

```python
import trendlib as tl
```

## 1. Batch functions

One lowercase function per indicator, named as in `INDICATORS.md`.

```python
tl.rsi(source, *, period=14)
tl.bbands(source, *, period=20, nbdev_up=2.0, nbdev_dn=2.0, ma_type="sma")
tl.atr(high, low, close, *, period=14)          # or tl.atr(df, period=14)
tl.vwap(high, low, close, volume, timestamps=None, *, anchor="day", tz="Asia/Kolkata")
```

### Arguments

- **Inputs are positional or keyword**, in the order `spec.yaml` lists them, using
  the input names (`source`, `open`, `high`, `low`, `close`, `volume`, `timestamps`).
- **Parameters are keyword-only**, named and defaulted exactly as in `spec.yaml`.
- **A DataFrame may replace all bar inputs** (pandas or polars). Columns are matched
  case-insensitively to `open`, `high`, `low`, `close`, `volume`. For
  single-series indicators a DataFrame uses its `close` column. Timestamps come from
  a pandas `DatetimeIndex`, or a column named `timestamp`/`datetime`/`date`/`time`
  if the index is not datetime-like. Missing columns raise `InvalidInput` naming them.
- **Array-likes:** NumPy arrays, pandas Series, polars Series, Python lists. All
  inputs must have equal length.
- **Enum parameters** take lowercase strings (`ma_type="ema"`); the uppercase aliases
  also accept TA-Lib's integers (`matype=1`).

### Return values

| Input type | Single output | Multi-output |
| --- | --- | --- |
| NumPy / list | `np.ndarray` | `tuple` of `np.ndarray`, in `spec.yaml` output order |
| pandas Series | `pd.Series` named after the output, same index | `pd.DataFrame`, one column per output, same index |
| pandas DataFrame | `pd.Series` (as above) | `pd.DataFrame` (as above) |
| polars Series / DataFrame | `pl.Series` named after the output | `pl.DataFrame` |

- Length always equals input length. Warm-up rows are `NaN` (float64 outputs) or
  `0` (int32 outputs). Rules in `CONVENTIONS.md`.
- Float outputs are `float64`; integer outputs (pattern flags, trend direction) are
  `int32`.
- Returned arrays are new objects; inputs are never modified.

### Errors

```python
class TrendLibError(Exception): ...
class InvalidInput(TrendLibError, ValueError): ...      # bad param, bad input, NaN/inf mid-series
class InsufficientHistory(TrendLibError, ValueError): ... # stream open with too few bars
```

Messages name the parameter or input, the bad value and the allowed range, e.g.
`rsi: period=1 is out of range [2, 100000]`.

## 2. TA-Lib aliases

For every indicator with a `talib:` block in its spec, an uppercase alias exists:

```python
tl.RSI(close, timeperiod=14)
upper, middle, lower = tl.BBANDS(close, timeperiod=20, nbdevup=2, nbdevdn=2, matype=0)
supertrend, trend = tl.SUPERTREND(high, low, close, timeperiod=10, multiplier=3.0)
```

Aliases take ta-lib-python's parameter names and integer enums, return ta-lib-python's
output order, and compute exactly what the lowercase function computes. They exist
for migration; docs and examples use lowercase functions.

## 3. Lookback

```python
tl.lookback("rsi", period=14)   # -> 14: number of NaN rows before the first value
```

Also available as `tl.registry.describe("rsi").lookback(period=14)`.

## 4. Streaming

```python
s = tl.stream.rsi(history, period=14)        # history: array-like or DataFrame, ≥ lookback+1 bars
s.value                                      # last committed value (float, int or tuple)
x = s.update(new_close)                      # commit one closed bar, return its value
y = s.peek(forming_close)                    # value if this bar closed now; state unchanged
fork = s.copy()                              # independent handle at the same bar
s.bars_seen                                  # bars consumed so far, history included

st = tl.stream.supertrend(df_history, period=10, multiplier=3.0)
st.update(high, low, close)                  # bar inputs positional, spec order
st.update(df_row)                            # or a mapping / pandas row with named fields

vw = tl.stream.vwap(df_history, anchor="day", tz="Asia/Kolkata")
vw.update(high, low, close, volume, timestamp)
```

- `tl.stream.<name>(history, **params)` opens a stream. Parameters are fixed for
  the stream's life; a new parameter means a new stream.
- Too little history raises `InsufficientHistory` stating how many bars are needed.
- A bar with NaN or inf raises `InvalidInput` and leaves the stream unchanged.
- Multi-output streams return a named tuple with the output names
  (`s.value.bbands_upper`).
- `tl.stream.<name>.open_and_fill(history, **params)` returns `(stream, batch_outputs)`
  in one pass; `batch_outputs` equals `tl.<name>(history, **params)`.
- Guarantee: for any history and any sequence of bars, the values from `update`
  equal the batch function's values on the concatenated series, bitwise.

## 5. Registry

```python
tl.registry.groups()                 # ["overlap", "momentum", ...] in display order
tl.registry.names(group=None)        # ["ad", "adosc", "adx", ...]
info = tl.registry.describe("bbands")
info.name, info.title, info.group, info.flags
info.inputs     # [Input(name="source", kind="series")]
info.params     # [Param(name="period", type="int", default=20, min=2, max=100000, doc=...), ...]
info.outputs    # [Output(name="bbands_upper", dtype="float64", plot="upper_band"), ...]
info.talib      # TalibAlias(name="BBANDS", params={...}, outputs=[...]) or None
info.lookback(period=20)
tl.registry.to_json()                # whole catalogue as JSON (for UIs and LLM tool schemas)
```

## 6. pandas accessor

```python
df.tl.rsi(period=14)                 # same as tl.rsi(df, period=14); returns Series/DataFrame
df = df.join(df.tl.bbands())         # multi-output -> DataFrame with output-named columns
```

Registered when `trendlib` is imported and pandas is installed. No accessor is
registered if pandas is absent.

## 7. Versions

`tl.__version__` (string). `tl.__version_info__` (tuple of ints).

## 8. Not in the public API

Anything starting with `_`, the `_core` extension module, and `trendlib.registry`
internals beyond the names above.
