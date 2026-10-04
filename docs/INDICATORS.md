# Indicators

The approved catalogue. Names, parameters, defaults, ranges and outputs here are
authoritative: copy them into `spec.yaml` exactly. Anything not here needs an
approved spec issue first.

TA-Lib facts below were read from ta-lib-python **0.8.1** (`talib.abstract`,
2026-10-04). Lookbacks are asserted against TA-Lib by tests, not listed here.

## 1. Naming rules

- Function names are lowercase snake_case; the folder name equals the function name.
- Single-series input is named `source`. Bar inputs use `open`, `high`, `low`,
  `close`, `volume`, `timestamps`.
- Output names are globally descriptive so DataFrames can be joined without
  clashes: a single output takes the function's name (`rsi`); multi-output names are
  prefixed (`bbands_upper`).
- TA-Lib alias parameter renames (D12):

| TrendLib | ta-lib-python |
| --- | --- |
| `period` | `timeperiod` |
| `fast_period` / `slow_period` / `signal_period` | `fastperiod` / `slowperiod` / `signalperiod` |
| `nbdev_up` / `nbdev_dn` | `nbdevup` / `nbdevdn` |
| `ma_type` | `matype` (int) |
| `fastk_period`, `slowk_period`, `slowd_period` | same names |
| `slowk_ma_type` / `slowd_ma_type` | `slowk_matype` / `slowd_matype` (int) |
| `multiplier` | `multiplier` |

`ma_type` values in v0.1: `"sma"`=0, `"ema"`=1, `"wma"`=2, `"dema"`=3,
`"tema"`=4 (TA-Lib's integers). Other TA-Lib MA types raise `InvalidInput`
("not implemented yet") until their indicator ships.

## 2. v0.1 set (27 functions)

Ranges are inclusive; `max` = 100000 for periods. "Any" = any finite float.
Oracle **T** = ta-lib-python 0.8.1, run by `scripts/oracle/talib_golden.py`.
Oracle **H** = human-transcribed reference values (see `TESTING.md` § 2).

### Overlap

| Function | Alias | Inputs | Parameters (default, range) | Outputs (alias outputs) | Oracle |
| --- | --- | --- | --- | --- | --- |
| `sma` | `SMA` | source | `period` 30 [1, max] | `sma` (real) | T |
| `ema` | `EMA` | source | `period` 30 [1, max] | `ema` (real) | T |
| `wma` | `WMA` | source | `period` 30 [1, max] | `wma` (real) | T |
| `dema` | `DEMA` | source | `period` 30 [1, max] | `dema` (real) | T |
| `tema` | `TEMA` | source | `period` 30 [1, max] | `tema` (real) | T |
| `bbands` | `BBANDS` | source | `period` 20 [2, max]; `nbdev_up` 2.0 any; `nbdev_dn` 2.0 any; `ma_type` "sma" | `bbands_upper`, `bbands_middle`, `bbands_lower` (upperband, middleband, lowerband) | T |
| `supertrend` | `SUPERTREND` | high, low, close | `period` 10 [2, max]; `multiplier` 3.0 [0, any] | `supertrend` float64, `supertrend_direction` int32 (supertrend, trend) | T |

### Momentum

| Function | Alias | Inputs | Parameters (default, range) | Outputs (alias outputs) | Oracle |
| --- | --- | --- | --- | --- | --- |
| `rsi` | `RSI` | source | `period` 14 [2, max] | `rsi` (real) | T |
| `macd` | `MACD` | source | `fast_period` 12 [2, max]; `slow_period` 26 [2, max]; `signal_period` 9 [1, max] | `macd`, `macd_signal`, `macd_hist` (macd, macdsignal, macdhist) | T |
| `stoch` | `STOCH` | high, low, close | `fastk_period` 5 [1, max]; `slowk_period` 3 [1, max]; `slowk_ma_type` "sma"; `slowd_period` 3 [1, max]; `slowd_ma_type` "sma" | `stoch_k`, `stoch_d` (slowk, slowd) | T |
| `adx` | `ADX` | high, low, close | `period` 14 [2, max] | `adx` (real) | T |
| `cci` | `CCI` | high, low, close | `period` 14 [2, max] | `cci` (real) | T |
| `roc` | `ROC` | source | `period` 10 [1, max] | `roc` (real) | T |
| `willr` | `WILLR` | high, low, close | `period` 14 [2, max] | `willr` (real) | T |

### Volatility

| Function | Alias | Inputs | Parameters | Outputs (alias outputs) | Oracle |
| --- | --- | --- | --- | --- | --- |
| `trange` | `TRANGE` | high, low, close | — | `trange` (real) | T |
| `atr` | `ATR` | high, low, close | `period` 14 [1, max] | `atr` (real) | T |
| `natr` | `NATR` | high, low, close | `period` 14 [1, max] | `natr` (real) | T |

### Volume

| Function | Alias | Inputs | Parameters | Outputs (alias outputs) | Oracle |
| --- | --- | --- | --- | --- | --- |
| `obv` | `OBV` | source, volume | — | `obv` (real) | T |
| `ad` | `AD` | high, low, close, volume | — | `ad` (real) | T |
| `adosc` | `ADOSC` | high, low, close, volume | `fast_period` 3 [2, max]; `slow_period` 10 [2, max] | `adosc` (real) | T |
| `vwap` | `VWAP` (anchor="none") | high, low, close, volume, timestamps (optional when anchor="none") | `anchor` "day" {day, none}; `tz` "Asia/Kolkata" (IANA name) | `vwap` (real) | T per session (§ 3.1) |

### Levels

| Function | Alias | Inputs | Parameters | Outputs | Oracle |
| --- | --- | --- | --- | --- | --- |
| `cpr` | — | high, low, close | — | `cpr_pivot`, `cpr_tc`, `cpr_bc` | H |
| `pivots_traditional` | — | high, low, close | — | `pivots_pp`, `pivots_r1`, `pivots_r2`, `pivots_r3`, `pivots_s1`, `pivots_s2`, `pivots_s3` | H |
| `pivots_camarilla` | — | high, low, close | — | `camarilla_r1` … `camarilla_r4`, `camarilla_s1` … `camarilla_s4` | H |

### Patterns

| Function | Alias | Inputs | Outputs (int32) | Oracle |
| --- | --- | --- | --- | --- |
| `cdl_doji` | `CDLDOJI` | open, high, low, close | `cdl_doji` (integer) | T |
| `cdl_engulfing` | `CDLENGULFING` | open, high, low, close | `cdl_engulfing` (integer) | T |
| `cdl_hammer` | `CDLHAMMER` | open, high, low, close | `cdl_hammer` (integer) | T |

## 3. Approved definitions for indicators that differ from or are absent in TA-Lib

### 3.1 `vwap` — session-anchored VWAP

```
tp_t   = (high_t + low_t + close_t) / 3
vwap_t = Σ_{i ∈ session(t), i ≤ t} tp_i · volume_i  /  Σ_{i ∈ session(t), i ≤ t} volume_i
```

- `session(t)`: bars sharing the local date of `timestamps[t]` in `tz`
  (`anchor="day"`), or all bars (`anchor="none"`). See `CONVENTIONS.md` § 7.
- Lookback 0. Before any volume has traded in the session the value is `NaN`
  (Deviation 2). A zero-volume bar after volume has traded leaves the value
  unchanged.
- Oracle: TA-Lib `VWAP` run separately on each session's slice of the synthetic
  intraday dataset; rows affected by Deviation 2 are excluded and the header says so.
- Flags: `overlap`, `path_dependent`, `requires_timestamps` (when anchored).

### 3.2 `cpr` — Central Pivot Range

Inputs are bars of the period the levels are built from (normally daily bars). The
value in row `t` uses bar `t − 1`: it is the range in force during period `t`.

```
cpr_pivot_t = (high_{t−1} + low_{t−1} + close_{t−1}) / 3
cpr_bc_t    = (high_{t−1} + low_{t−1}) / 2
cpr_tc_t    = 2 · cpr_pivot_t − cpr_bc_t
```

- Lookback 1. `cpr_tc` is below `cpr_bc` when the prior close is below the prior
  midpoint; outputs are not reordered. `doc.md` says so.
- Reference: Frank Ochoa, *Secrets of a Pivot Boss*, Wiley, 2010.
- Intraday use with timestamps (deriving the prior day from intraday bars) is M6.

### 3.3 `pivots_traditional` — floor pivots

Row `t` uses bar `t − 1` (H, L, C below). Lookback 1.

```
pp = (H + L + C) / 3
r1 = 2·pp − L          s1 = 2·pp − H
r2 = pp + (H − L)      s2 = pp − (H − L)
r3 = H + 2·(pp − L)    s3 = L − 2·(H − pp)
```

Should match TradingView "Pivot Points Standard" with type **Traditional** (levels
1–3). If transcribed TradingView values disagree with these formulas, stop and ask;
do not adjust formulas to fit.

### 3.4 `pivots_camarilla`

Row `t` uses bar `t − 1`. Lookback 1. With `R = H − L`:

```
camarilla_r4 = C + 1.1·R/2     camarilla_s4 = C − 1.1·R/2
camarilla_r3 = C + 1.1·R/4     camarilla_s3 = C − 1.1·R/4
camarilla_r2 = C + 1.1·R/6     camarilla_s2 = C − 1.1·R/6
camarilla_r1 = C + 1.1·R/12    camarilla_s1 = C − 1.1·R/12
```

Reference: Nick Scott's Camarilla equation (1989). Should match TradingView "Pivot
Points Standard", type **Camarilla**, levels 1–4.

### 3.5 `supertrend`

Exactly TA-Lib's SUPERTREND (bands carried forward on every bar; initial trend
seeded up; direction +1 up). The only renamed output is `trend` →
`supertrend_direction`. `doc.md` must state the TradingView sign and seed
difference (`CONVENTIONS.md` § 10).

## 4. Build order for M4

Dependencies first; each line can be one PR.

1. `wma`, `trange`
2. `dema`, `tema` (EMA from M1)
3. `bbands` (needs `ma_type` dispatch over sma/ema/wma/dema/tema)
4. `macd`, `roc`, `willr`, `cci`
5. `atr`, `natr`
6. `adx`
7. `stoch`
8. `obv`, `ad`, `adosc`
9. `supertrend`
10. `vwap`
11. `cpr`, `pivots_traditional`, `pivots_camarilla`
12. `cdl_doji`, `cdl_engulfing`, `cdl_hammer`

## 5. Backlog (v1.0, spec issue needed for each)

| Group | Candidates |
| --- | --- |
| Overlap | kama, t3, trima, hma, zlema, rma, kc (Keltner), donchian, ichimoku, vwma |
| Momentum | mfi, aroon, aroonosc, ppo, trix, ultosc, cmo, stochrsi, apo, mom |
| Volatility | stddev, historical volatility, chaikin volatility |
| Volume | cmf |
| Levels | pivots_fibonacci, opening_range, intraday cpr (timestamp-anchored), weekly/monthly anchors |
| India data | oi_change, pcr (put-call ratio), delivery_ratio (delivered qty ÷ traded qty) — needs an input-data contract first |
| Patterns | remaining TA-Lib candlesticks (61 total), configurable candle settings |
