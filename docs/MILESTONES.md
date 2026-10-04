# Milestones

Build one milestone at a time, on branch `m<N>-<slug>`, and stop at the end with the
report format from `CLAUDE.md`. A milestone is done only when every acceptance
check passes, or a human accepts the listed exceptions.

| # | Name | Result |
| --- | --- | --- |
| M0 | Scaffold | Empty but real package that builds, imports and passes CI on 3 OSes |
| M1 | First indicators end to end | SMA, EMA, RSI with batch, stream, Python, golden + parity + edge tests |
| M2 | Code generator | Specs drive registry, bindings, wrappers, stubs, docs, generated tests |
| M3 | Packaging + TestPyPI | Wheels for all platforms published by CI; `0.1.0a1` on PyPI |
| M4 | Core indicator set | The 27 functions in `INDICATORS.md` § 2.1 |
| M5 | Public 0.1.0 | Docs site, README, notebooks, release |
| M6 | Full TA-Lib parity | The remaining 174 functions in `INDICATORS_TALIB.md` |
| M7 | v1.0 | Beyond-TA-Lib list, NSE/BSE calendar, crates.io, API freeze |

---

## M0 — Scaffold

Tasks:

- [ ] Cargo workspace: `crates/trendlib` (lib, `#![forbid(unsafe_code)]`, no deps),
      `crates/trendlib-py` (cdylib, pyo3 + numpy, `publish = false`),
      `crates/xtask` (bin, `publish = false`), `.cargo/config.toml` alias
      `xtask = "run -p xtask --"`. Shared `[workspace.package]` version `0.0.0`.
- [ ] `rust-toolchain.toml` (stable, pinned minor), `deny.toml`, `.editorconfig`,
      `.gitignore` (target, dist, `.venv`, `__pycache__`, `*.so`, `*.pyd`).
- [ ] `pyproject.toml` exactly as `RELEASE.md` § 2.
- [ ] `python/trendlib/__init__.py` exposing `__version__` and `__version_info__`
      from `_core`; `errors.py`; `py.typed`.
- [ ] `trendlib._core` module with a `__version__` attribute and nothing else yet.
- [ ] `xtask` with subcommands `generate`, `regen-check`, `golden`, `bench` that
      exit non-zero with "not implemented until M<k>".
- [ ] `scripts/testdata/make_synthetic.py` and the two datasets in `TESTING.md` § 3,
      committed.
- [ ] `ci.yml` jobs `lint`, `rust`, `python` from `TESTING.md` § 9.
- [ ] `LICENSE-MIT`, `LICENSE-APACHE` (copyright "GAIIN Technologies Private
      Limited and TrendLib contributors"), `CODE_OF_CONDUCT.md` (Contributor
      Covenant 2.1), `SECURITY.md` (GitHub private vulnerability reporting),
      `CHANGELOG.md` (Keep a Changelog, "Unreleased"). Keep the provided
      `README.md` and `CONTRIBUTING.md`; fill in only what M0 makes true.
- [ ] `.github/pull_request_template.md` and issue templates (provided) kept.

Acceptance:

- [ ] `maturin develop --release` succeeds; `python -c "import trendlib; print(trendlib.__version__)"` prints `0.0.0`.
- [ ] `cargo test --workspace`, `cargo clippy ... -D warnings`, `cargo fmt --check`, `pytest` all pass locally.
- [ ] CI green on ubuntu, macOS and windows.
- [ ] `cargo tree -p trendlib -e normal` shows no dependencies.

## M1 — First indicators end to end

Tasks:

- [ ] `core/`: traits (`ARCHITECTURE.md`), `TlError`, input views and validation
      (leading-NaN detection, finiteness, equal lengths), aligned output buffers,
      shared kernels needed by SMA/EMA/RSI.
- [ ] `indicators/sma`, `ema`, `rsi`: `spec.yaml` (from `INDICATORS.md`), `mod.rs`,
      `doc.md`, `golden/default.csv` + `golden/min_period.csv`.
- [ ] `scripts/oracle/talib_golden.py` and `cargo xtask golden <name>` calling it.
- [ ] Hand-written PyO3 bindings for the three (batch functions + stream classes)
      and Python wrappers following `PYTHON_API.md` for NumPy and pandas
      (polars may wait for M2). These are replaced by generated code in M2.
- [ ] `tl.lookback`, `tl.stream.<name>`, `open_and_fill`, aliases `SMA`/`EMA`/`RSI`.
- [ ] `tests/golden.rs`, a hand-written parity test and edge-case test for the
      three (generated in M2), Python contract + TA-Lib parity tests.

Acceptance:

- [ ] Golden tests pass for all cases; each was shown to fail on a deliberately
      broken implementation (state how in the report).
- [ ] Stream equals batch bitwise in the property test (≥ 1,000 cases each).
- [ ] `tl.rsi`, `tl.sma`, `tl.ema` agree with ta-lib-python within 1e-10 relative on
      the property test; lookbacks equal TA-Lib's.
- [ ] Every edge case in `TESTING.md` § 5 behaves as specified for the three.
- [ ] pandas in → pandas out with the same index and output-named Series.
- [ ] Update `ARCHITECTURE.md` if the traits changed.

## M2 — Code generator

Tasks:

- [ ] `cargo xtask generate` per `ARCHITECTURE.md` § Code generator, with strict
      spec validation (`SPEC_FORMAT.md`).
- [ ] Replace the M1 hand-written bindings, wrappers and per-indicator parity/edge
      tests with generated ones; delete the hand-written versions.
- [ ] `_convert.py` with full NumPy / pandas / polars handling; `pandas_ext.py`;
      `registry.py` incl. `to_json()`.
- [ ] Generated docs pages `docs/indicators/<name>.md` + index; `mkdocs.yml` that
      builds them, with `pymdownx.arithmatex` for `$$` formulas (site publishing
      is M5).
- [ ] `api/public_api.txt` generated; CI jobs `regen` and `api`.
- [ ] Spec-validation tests: a fixture folder of broken specs, each failing with the
      expected message.

Acceptance:

- [ ] `cargo xtask generate` twice in a row → `git status` clean.
- [ ] Deleting any generated file and regenerating restores it byte for byte.
- [ ] All M1 tests pass against generated code; polars tests pass.
- [ ] Adding a dummy indicator folder (in a scratch branch) produces a working
      function, stream, stub, docs page and tests with no other hand edits.
- [ ] `mkdocs build --strict` succeeds.

## M3 — Packaging and TestPyPI

Prerequisite (human): `RELEASE.md` § 4 one-time setup, and Q1 answered.

Tasks:

- [ ] `release.yml` per `RELEASE.md` § 3–4, including both smoke-test jobs and the
      tag/version check.
- [ ] CI job that builds the sdist and installs it in a clean environment.
- [ ] `cargo publish --dry-run -p trendlib` in CI.
- [ ] Bump to `0.1.0-alpha.1`.

Acceptance:

- [ ] A `workflow_dispatch` run publishes to TestPyPI and the smoke test passes on
      ubuntu, macOS and windows.
- [ ] Wheels exist for every row of `RELEASE.md` § 3 and are `abi3`.
- [ ] sdist installs from source with only Rust and Python present.
- [ ] After human approval: `0.1.0a1` on PyPI and `pip install --pre trendlib` works.

## M4 — Core indicator set

The 27 functions in `INDICATORS.md` § 2.1. They are not an arbitrary 27: between
them they are the first to exercise every shape the generator and the test
harness have to handle — multi-input (`atr`), multi-output (`bbands`, `macd`),
an enum parameter (`bbands`), an int32 output (`cdl_*`), path dependence
(`supertrend`, `obv`), timestamps and sessions (`vwap`), and an oracle that has
to be transcribed by a human (`cpr`, the pivots). Anything that breaks on the
other 174 functions should break here first, on a set small enough to debug.

Tasks:

- [ ] The remaining 24 functions, in the order of `INDICATORS.md` § 4 stage 1,
      one PR-sized commit series each, using the `new-indicator` skill.
- [ ] `time/` support for `vwap` (`CONVENTIONS.md` § 7), timestamp conversion in
      `_convert.py`.
- [ ] Benchmarks (`TESTING.md` § 8) and the nightly workflow.

Acceptance:

- [ ] Every function has `spec.yaml`, `mod.rs`, `doc.md`, golden files and passes
      golden, parity and edge suites. H-oracle goldens may be `#[ignore]` pending Q4,
      listed in the report.
- [ ] TA-Lib parity property tests pass for all T-oracle functions.
- [ ] No shared indicator slower than 1.5× TA-Lib in the benchmark table, or each
      exception listed with a reason.
- [ ] Adding one of the 174 remaining functions needs no change outside its own
      folder and the generated files. Demonstrate on one function from a group
      M4 did not touch, in a scratch branch.

## M5 — Public 0.1.0

Tasks:

- [ ] Docs site on GitHub Pages (`docs.yml`): home, install, quickstart, API
      reference (generated), conventions, migrating from TA-Lib, contributing.
- [ ] README quickstart verified by a doctest-style test.
- [ ] `examples/`: NIFTY-style Supertrend on daily bars, session VWAP on 5-minute
      bars, live streaming loop — all on the synthetic datasets.
- [ ] Release `0.1.0` per `RELEASE.md` § 5.

Acceptance:

- [ ] `pip install trendlib` (no `--pre`) works on fresh ubuntu, macOS, windows.
- [ ] Docs site live; every page builds with `--strict`.
- [ ] Notebooks execute top to bottom in CI.

## M6 — Full TA-Lib parity

Every remaining function in `INDICATORS_TALIB.md`: 201 from the catalogue minus
the 24 that M4 already shipped, in the order of `INDICATORS.md` § 4 stage 2.
Released incrementally as 0.2, 0.3 and so on, a group at a time, rather than as
one long branch.

This milestone is only tractable because M2 generates the registry, bindings,
wrappers, stubs, docs pages and the parity and edge-case suites. The work per
function is `spec.yaml`, `mod.rs`, `doc.md` and golden data from the oracle;
everything else is generated. If that stops being true, stop and fix the
generator rather than hand-writing the difference.

Tasks:

- [ ] Shared kernels before the functions that use them: the MA dispatch table,
      the candle-settings kernel for the 58 remaining patterns, and the
      Hilbert-transform kernel for the cycle group.
- [ ] Each group from `INDICATORS.md` § 4 stage 2, with its own release.
- [ ] Benchmarks extended to every shared indicator (`TESTING.md` § 8).

Acceptance:

- [ ] `tl.registry.names()` lists all 204 functions, and every one has
      `spec.yaml`, `mod.rs`, `doc.md`, golden files and passes the golden,
      parity and edge-case suites.
- [ ] For every function with a `talib:` block, the property-based parity suite
      passes and `tl.lookback(...)` equals TA-Lib's lookback.
- [ ] `python scripts/catalogue/talib_catalogue.py --check` is clean and every
      catalogue row has a folder.
- [ ] No shared indicator slower than 1.5× TA-Lib, or each exception listed.

## M7 — v1.0

Scope: the beyond-TA-Lib list in `INDICATORS.md` § 5 once it is approved, the
NSE/BSE trading calendar (F13), per-call candle settings, the Q2 decision,
the crates.io publish, and the public API frozen in `api/public_api.txt`. Plan
this milestone with a human before starting; split it into numbered
sub-milestones in this file first.
