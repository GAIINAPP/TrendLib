<div align="center">

# TrendLib

**325 technical-analysis indicators and patterns. Rust core, Python API.**

[![CI](https://github.com/GAIINAPP/TrendLib/actions/workflows/ci.yml/badge.svg)](https://github.com/GAIINAPP/TrendLib/actions/workflows/ci.yml)
[![Python](https://img.shields.io/badge/python-3.11%20%7C%203.12%20%7C%203.13%20%7C%203.14-blue)](https://www.python.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green)](#license)

Every indicator runs two ways: over a whole array, or one bar at a time.
The two agree **bitwise**, so what you backtest is what you trade.

</div>

---

```python
import numpy as np
import trendlib as tl

close = np.array([...])

tl.rsi(close, period=14)                  # one array out
macd, signal, hist = tl.macd(close)       # three
tl.cdl_engulfing(open_, high, low, close) # int32: +100, -100 or 0
```

## Install

```bash
pip install trendlib
```

The first release ships a source distribution, so this compiles the Rust core
and needs a toolchain (1.95 or later). Nothing else is required: no system
TA-Lib, no C dependencies.

Pre-built wheels are on the way. The extension is `abi3`, so one wheel per
platform will cover Python 3.11 and every version after it, and the install
becomes a download with no toolchain at all.

## What's in it

| Group | Count | Examples |
| --- | ---: | --- |
| Chart patterns | 63 | head and shoulders, double top, wedges, channels, flags, cup with handle |
| Candlestick patterns | 61 | engulfing, harami, morning star, three black crows, hikkake |
| Momentum | 51 | RSI, MACD, stochastic, ADX, CCI, Williams %R, Connors RSI, TSI |
| Overlap studies | 34 | SMA, EMA, WMA, HMA, KAMA, T3, Bollinger, Keltner, Ichimoku, SuperTrend |
| Statistics | 23 | linear regression, correlation, beta, standard deviation, percentile |
| Bar patterns | 19 | inside day, outside day, key reversal, pipe top, narrow range |
| Volume | 16 | OBV, A/D, Chaikin, MFI, VWAP, VWMA, Twiggs money flow |
| Math transforms | 15 | log, exp, trigonometric and hyperbolic functions |
| Harmonic patterns | 12 | Gartley, bat, butterfly, crab, ABCD, Wolfe wave |
| Volatility | 9 | ATR, NATR, true range, Chaikin volatility, GAPO |
| Levels | 7 | traditional, Camarilla, Fibonacci, Woodie and DeMark pivots, CPR |
| Cycle | 6 | Hilbert transform: dominant cycle period, phase, trend mode |
| Math operators | 5 | add, subtract, multiply, divide, cumulative sum |
| Price transforms | 4 | typical, weighted, median and average price |

201 of them carry a TA-Lib-style uppercase alias, so `tl.RSI(close, timeperiod=14)`
works beside `tl.rsi(close, period=14)` and takes TA-Lib's parameter names.

## How it works

Three layers, each doing one job.

**A Rust core** holds the arithmetic. It has no runtime dependencies at all
(`cargo tree` prints one line) and is `#![forbid(unsafe_code)]`.

**One kernel per indicator**, written once as a step that takes a bar and
returns the next value. The batch function folds that step over an array and the
stream calls it per bar, so they cannot drift apart: the bitwise agreement is
structural, not something a test happens to confirm.

**A thin Python layer** does the conversion: NumPy in, NumPy out, with pandas
and polars recognised and their index handed back.

### Streaming

Streams are values, not handles. Each one is independent, nothing is shared, and
there are no global settings anywhere.

```python
live = tl.stream.rsi(history)   # seed with what you have
live.update(bar)                # advance, and get the new value
live.peek(bar)                  # what it would read, without committing
live.copy()                     # fork the state to explore a branch
live.value, live.bars_seen      # where it stands
```

`tl.stream.<name>` exists for all 325.

### Frames

Pass a DataFrame and the bar columns are matched by name, case-insensitively:

```python
tl.atr(frame)                      # finds high, low, close
tl.rsi(frame["close"])             # a Series comes back indexed like it went in
```

### Warm-up

Every output starts where the indicator is first defined. Earlier rows are
`NaN`, or `0` for the integer columns patterns return, and the arrays always
come back the same length as the input.

```python
tl.lookback("rsi", period=14)      # 14 - the first row that carries a value
```

### Errors

```python
tl.TrendLibError          # the base
tl.InvalidInput           # wrong shape, wrong dtype, NaN mid-series
tl.InsufficientHistory    # fewer bars than the lookback needs
```

Nothing is silently clamped or filled. A parameter outside its range raises
rather than snapping to the nearest legal value.

## Design notes

Measurements, not advice. No function, output or parameter is named for a
trading action: there is no `buy`, `entry` or `target` anywhere. Pattern
functions return `+100`, `-100` or `0`, meaning *this shape is present and
points up, points down, or is absent*; what to do about it is yours.

Where a published formula and a widely used implementation disagree, the
division and ordering are chosen to keep precision rather than to match an
implementation quirk. Several indicators here are measurably closer to exact
arithmetic than the reference libraries they are compared against.

## Platforms

| | |
| --- | --- |
| Python | 3.11 and later (`abi3`) |
| Rust | 1.95, edition 2024 |
| OS | Linux (glibc, musl), macOS, Windows |
| Arch | x86-64, aarch64 |

## License

[Apache License 2.0](LICENSE). It grants patent rights explicitly, which a
permissive licence that is silent on patents does not.

Built by [GAIIN Technologies](https://github.com/GAIINAPP).
