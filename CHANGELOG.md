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
