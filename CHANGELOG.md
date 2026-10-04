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
