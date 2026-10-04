# TrendLib — Product Requirements

Status: approved for build, v0.1 scope. Owner: GAIIN Technologies.

TrendLib is a fast, correct, pip-installable technical-indicator library with a
Rust core, a Python-first API, a live streaming API that matches batch output
exactly, and Indian-market indicators built in.

## 1. Problem

Indian quant developers, fintech teams and retail algo traders combine TA-Lib, which
has a C-style API and no session awareness, with slower pure-Python libraries whose
warm-up rules are inconsistent. They also hand-roll session VWAP, CPR and pivot code
that rarely matches what broker charts show, and live and backtest values often
disagree. GAIIN's own platform needs one trusted indicator engine.

## 2. Goals

1. **Correct.** Every indicator is validated against an independent, executable
   oracle with a documented tolerance.
2. **Fast.** Rust core; batch is O(n); streaming update is O(1) per bar for
   fixed-window indicators; no allocation per streaming update.
3. **Easy.** `pip install trendlib` works on Linux, macOS and Windows without a
   compiler; NumPy, pandas and polars in and out.
4. **Consistent.** Streaming output equals batch output bitwise on every bar.
5. **India-ready.** Session-anchored indicators work on NSE/BSE intraday data out of
   the box (IST sessions); CPR and pivot levels as used by Indian traders.
6. **Discoverable.** Machine-readable metadata (names, params, ranges, outputs,
   plot hints) for UIs, docs and AI tool-calling.

## 3. Non-goals (v1)

Data feeds or broker connectors · backtesting · strategy logic, signals or
recommendations · charting · GPU · decimal arithmetic (float64 only).

## 4. Users

| Persona | Needs | Primary surface |
| --- | --- | --- |
| Quant / algo developer | Accurate batch + streaming, speed, TA-Lib parity | Python API |
| Fintech engineer (incl. GAIIN) | Stable API, metadata for indicator pickers, live feeds | Python, later Rust crate |
| Retail coder learning algo trading | One-line install, clear docs, Indian-market examples | Docs + notebooks |
| Contributor | Clear spec format, fast tests, small PRs | Repo + `CONTRIBUTING.md` |

## 5. Functional requirements

| ID | Requirement | Pri | Milestone |
| --- | --- | --- | --- |
| F1 | Each indicator is defined by `spec.yaml` + `mod.rs` + `doc.md` + golden CSVs in one folder | P0 | M1 |
| F2 | Batch API returns outputs aligned to input length with NaN warm-up (`CONVENTIONS.md`) | P0 | M1 |
| F3 | Streaming API: open from history, `update`, `peek`, `value`, `copy` | P0 | M1 |
| F4 | `lookback(**params)` available for every indicator | P0 | M1 |
| F5 | NumPy, pandas and polars inputs; same type out; pandas index preserved; OHLCV columns auto-mapped | P0 | M2 |
| F6 | Registry: list groups and indicators, describe params (type, default, range), outputs and plot hints, as Python objects and JSON | P0 | M2 |
| F7 | Uppercase TA-Lib aliases with ta-lib-python parameter and output names for every shared indicator | P1 | M2 (as indicators land) |
| F8 | v0.1 indicator set incl. India set: session VWAP, Supertrend, CPR, classic and Camarilla pivots (`INDICATORS.md`) | P0 | M4 |
| F9 | Candlestick patterns returning int32 flags (+100 / −100 / 0) | P1 | M4 |
| F10 | Docs site generated from specs: signature, params table, formula, example | P0 | M2 + M5 |
| F11 | Wheels on PyPI for all supported platforms; sdist builds with only Rust installed | P0 | M3 |
| F12 | Rust crate on crates.io | P1 | M6 |
| F13 | NSE/BSE trading calendar (holidays, sessions) for opening-range and multi-day anchors | P1 | M6 |

## 6. Non-functional requirements

| Area | Requirement |
| --- | --- |
| Accuracy | ≤ 1e-10 relative error vs oracle (abs 1e-12 near zero); looser only with a written reason in the golden header |
| Batch vs stream | Bitwise identical on the same platform, every indicator, every bar |
| Performance | Batch within 1.5× of TA-Lib C (via ta-lib-python) on 1,000,000 bars for shared indicators; GIL released during compute |
| Platforms | Wheels: Linux x86_64 + aarch64 (manylinux and musllinux), macOS x86_64 + arm64, Windows x86_64; Python ≥ 3.11 (abi3) |
| Edge cases | Defined behaviour for empty input, short input, constant series, leading NaN, NaN/inf mid-series, extreme magnitudes |
| Thread safety | No global mutable state; streams are independent values |
| Stability | SemVer; public API snapshot in `api/public_api.txt` checked in CI |
| Dependencies | Core crate: zero runtime dependencies. Python package: `numpy` only (pandas, polars optional) |
| Security | `#![forbid(unsafe_code)]` in core; `cargo deny` in CI; private vulnerability reporting enabled |

## 7. Scope

v0.1 = 27 functions listed in `INDICATORS.md` § 2. v1.0 backlog in
`INDICATORS.md` § Backlog. Milestones in `MILESTONES.md`.

## 8. Success metrics (first 12 months)

- 60+ indicators, each with an executed-oracle golden test.
- No correctness bug left open more than 14 days.
- Install succeeds on every CI platform for every release.
- GAIIN's platform computes all its indicators with TrendLib.
- Stars, monthly PyPI downloads and external contributors tracked monthly (targets set after 0.1.0).

## 9. Risks

| Risk | Mitigation |
| --- | --- |
| Values differ from broker charts and users call it a bug | Every `doc.md` states its conventions and known differences vs TradingView; Q2 compat mode |
| No executable oracle for India-specific indicators | Human-transcribed reference values with recorded provenance (`TESTING.md`) |
| Maintainer bandwidth | Spec-first issues, small PRs, AI-assisted implementation with human-reviewed oracles |
| Regulatory perception | Descriptive outputs only (D11) |
| TA-Lib already covers much of the classic set | Differentiate on API ergonomics, DataFrame support, streaming parity, session awareness, metadata, Rust crate |
