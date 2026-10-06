# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Before 1.0, minor versions may break the API; every break is listed under
"Changed (breaking)".

## [Unreleased]

### Added

- Cargo workspace with the core crate `trendlib` (no runtime dependencies,
  `#![forbid(unsafe_code)]`), the PyO3 bindings crate `trendlib-py`, and `xtask`.
- Python package `trendlib` exposing `__version__`, `__version_info__` and the
  error hierarchy `TrendLibError` / `InvalidInput` / `InsufficientHistory`.
- `scripts/testdata/make_synthetic.py` and the committed synthetic datasets
  `testdata/daily_2000.csv` and `testdata/intraday_5m_20d.csv`.
- CI on Linux, macOS and Windows: format, clippy, `cargo deny`, Rust tests and
  Python tests on 3.11 and 3.14.
- Indicators `sma`, `ema` and `rsi`, each with batch and streaming forms that
  agree bar for bar, NumPy and pandas input and output, and golden data produced
  by running ta-lib-python.
- `tl.lookback(name, **params)`, `tl.stream.<name>(history, **params)` with
  `update`, `peek`, `value`, `copy`, `bars_seen` and `open_and_fill`, and the
  TA-Lib-style aliases `tl.SMA`, `tl.EMA`, `tl.RSI`.
- `cargo xtask golden <name>` regenerates an indicator's golden files from its
  oracle.
- Indicators `wma`, `trange`, `atr` and `natr` in the Rust core, with golden,
  parity and edge-case coverage. They reach the Python API when the M2
  generator writes the bindings.
- `cargo xtask generate` writes the indicator module list, the PyO3 bindings,
  the Python wrappers and aliases, the stream factories and the type stubs from
  every `spec.yaml`; `cargo xtask regen-check` fails when any of them is stale.
  Adding an indicator is now its own folder plus generated files.
- `wma`, `trange`, `atr` and `natr` are callable from Python.
- Indicators `dema`, `tema`, `roc` and `macd`. `macd` is the first with
  several outputs, returned as a tuple or, from a DataFrame, as a frame.
- `maxindex`, `minindex`, `minmaxindex`, `aroon` and `aroonosc`.
- Row-index outputs carry an `absolute_index` flag: they index the caller's own
  array, so skipping leading warm-up rows shifts them.
- The directional movement family: `plus_di`, `minus_di`, `dx`, `adx`, `adxr`.
- `plus_di` and `minus_di` stay a percentage at `period = 1`, where TA-Lib
  returns the raw fraction (`CONVENTIONS.md` deviation 7).
- 6 more indicators: `bop`, `rma`, `trima`, `plus_dm`, `minus_dm` and `cmo`.
- 13 more indicators: `mom`, `rocp`, `rocr`, `rocr100`, `midpoint`, `midprice`,
  `sum`, `avgdev`, `stddev`, `var`, `max`, `min` and `minmax`. `stddev` and
  `var` are the first with a float parameter.
- 24 more indicators: the four price transforms, the fifteen math transforms
  and the five arithmetic operators.
- A DataFrame is refused where it cannot say which column is which operand,
  instead of quietly using `close` twice.
- `cci` and `wma` rebuild their window each bar rather than carrying a running
  total, which the measured drift required.
- Indicators `willr`, `cci`, `obv` and `ad`. `obv` and `ad` are the first
  path-dependent ones and the first to reject a negative volume.
- The Rust test registry is generated too, so every indicator is covered by
  the golden, parity and edge-case suites the moment its spec exists.
- `docs/PROGRESS.md` tracks all 204 approved indicators, ticked from the
  repository rather than by hand.
- `natr` normalises at every period, including `period = 1` where TA-Lib
  returns the raw true range instead (`CONVENTIONS.md` deviation 6).
- Nineteen chart patterns, the first beyond-TA-Lib group
  (`docs/INDICATORS.md` section 5.1): `chart_double_top`,
  `chart_double_bottom`, `chart_triple_top`, `chart_triple_bottom`,
  `chart_head_shoulders`, `chart_inverse_head_shoulders`, `chart_rising_wedge`,
  `chart_falling_wedge`, `chart_ascending_triangle`,
  `chart_descending_triangle`, `chart_symmetrical_triangle`,
  `chart_broadening`, `chart_bull_flag`, `chart_bear_flag`,
  `chart_bull_pennant`, `chart_bear_pennant`, `chart_rectangle`,
  `chart_ascending_channel` and `chart_descending_channel`. Each takes `high`,
  `low` and `close` and reads `+100`, `-100` or `0`, with a stream that agrees
  with batch bar for bar. Golden data comes from running `ta-patterns` 1.2.1
  (oracle P); `testdata/charts_2579.csv` holds shapes that make every one fire
  in each direction it reads. Where TrendLib and that oracle differ is
  `CONVENTIONS.md` deviations 8 and 9.
- `chart_cup_with_handle`, `chart_inverted_cup_with_handle` (oracle P) and
  `ichimoku` (oracle A, through TA-Lib's `MIDPRICE`): the first of the 105
  functions `docs/INDICATORS.md` §§ 5.2 to 5.4 approve.
- Nineteen bar patterns on oracle P (`docs/INDICATORS.md` § 5.2): 2B, 1-2-3,
  the two-bar gap, key, hook, one-day and pivot-point reversals, outside and
  inside day, fakey, wide-ranging day, NR4, NR7, pipe and horn tops and
  bottoms, and the dead-cat bounce and its inverse.
- Twenty-one more chart patterns on oracle P: rounding top and bottom,
  diamond top and bottom, bump-and-run top and bottom, island top and bottom,
  V-top and V-bottom, the complex head and shoulders both ways, the high and
  tight flag, measured moves up and down, the two broadening wedges, the two
  right-angled broadening formations and the two scallops the oracle can
  fire. `testdata/shapes_1640.csv` holds the shapes no walk draws.
- Twelve more on oracle P: the Adam and Eve double tops and bottoms (eight,
  graded the way Bulkowski names them), Big M and Big W, and three falling
  peaks and three rising valleys.
- Nine busted patterns on oracle P: the ascending and descending triangle,
  double and triple top and bottom, head and shoulders both ways, and the
  rectangle, each read when price reverses soon after its base pattern does.
- Twelve harmonic patterns on oracle P: AB=CD, Gartley, Bat, Butterfly, Crab
  and the Wolfe wave, each bullish and bearish.
- Four indicators on oracle F (`finta` 1.3, test-only, `docs/DECISIONS.md`
  D19): `wavetrend`, `ift_rsi`, `vzo` and `pivots_fibonacci`. Their averages
  follow pandas' `ewm(adjust=True)` recursion, as the oracle's do.
