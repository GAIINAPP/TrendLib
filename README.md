# TrendLib

Fast, tested technical-analysis indicators for Python, with a Rust core.

> **Status: under construction.** Nothing is published yet. This README describes the
> 0.1 target; sections become true as milestones land (`docs/MILESTONES.md`).

```bash
pip install trendlib
```

```python
import trendlib as tl

df["rsi"] = tl.rsi(df["close"], period=14)          # same length, NaN warm-up, same index
bands = tl.bbands(df, period=20)                      # DataFrame: bbands_upper, bbands_middle, bbands_lower
vwap = tl.vwap(df, anchor="day", tz="Asia/Kolkata")   # session VWAP, resets each trading day

s = tl.stream.rsi(df["close"], period=14)             # live feed: O(1) per bar
latest = s.update(new_close)                          # equals tl.rsi on the extended series, bitwise
```

## Why TrendLib

- **Correct by evidence.** Every indicator is checked against an independent,
  executed reference (TA-Lib for the classic set), with the tolerance in the repo.
- **Batch and live agree.** Streaming values equal batch values bit for bit.
- **Works with your data.** NumPy, pandas and polars in; the same type out, index kept.
- **Built for Indian markets too.** Session-anchored VWAP on IST sessions, CPR,
  traditional and Camarilla pivots, Supertrend.
- **Easy to migrate.** `tl.RSI(close, timeperiod=14)` and other TA-Lib-style aliases
  return what TA-Lib returns.
- **Self-describing.** `tl.registry.describe("bbands")` lists params, ranges and
  outputs; `tl.registry.to_json()` feeds UIs and tool schemas.

## Indicators (0.1)

Overlap: SMA, EMA, WMA, DEMA, TEMA, Bollinger Bands, Supertrend ·
Momentum: RSI, MACD, Stochastic, ADX, CCI, ROC, Williams %R ·
Volatility: True Range, ATR, NATR ·
Volume: OBV, A/D, A/D Oscillator, VWAP ·
Levels: CPR, traditional pivots, Camarilla pivots ·
Patterns: Doji, Engulfing, Hammer.

## Using your own data

TrendLib takes arrays and frames, not connections. Hand it a DataFrame with
`open`/`high`/`low`/`close`/`volume` and a timezone-aware index, one instrument
at a time, and every function works. `docs/DATA_INTEGRATION.md` has the full
contract, ready-made loaders for daily and intraday SQL tables, how to continue
a live stream from stored history, and the mistakes that produce wrong numbers
instead of errors.

## Documentation

Docs site: _coming in 0.1_. Until then: `docs/PYTHON_API.md`, `docs/CONVENTIONS.md`,
`docs/INDICATORS.md`, `docs/DATA_INTEGRATION.md`.

TrendLib computes measurements. It does not give investment advice.

## Contributing

See `CONTRIBUTING.md`. New indicators start as a spec issue.

## License

Dual-licensed under MIT or Apache-2.0, at your option. Built by GAIIN Technologies.
