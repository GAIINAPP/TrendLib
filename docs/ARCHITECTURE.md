# Architecture

One repo with a Cargo workspace and a maturin-built Python package at the root.
Humans write one folder per indicator. A generator derives everything repetitive
from those folders, and tests prove the result against independent oracles.

```
 crates/trendlib/src/indicators/<name>/        (hand-written)
   spec.yaml ─┐   mod.rs ─┐   doc.md ─┐   golden/*.csv ─┐
              │           │           │                 │
              ▼           │           ▼                 │
      cargo xtask generate│     docs pages              │
     ┌───────┬────────────┼──────────┬─────────┐        │
     ▼       ▼            ▼          ▼         ▼        ▼
 registry  mod.rs     generated.rs  _functions.py   generated tests
 (Rust)  (module list) (PyO3)       + _core.pyi     (parity, edge cases, golden)
```

## Repository layout

```
trendlib/
├── pyproject.toml                  PyPI package "trendlib" (maturin backend)
├── Cargo.toml                      workspace; [workspace.package] version shared by all crates
├── rust-toolchain.toml             stable channel, pinned minor
├── deny.toml                       cargo-deny: licenses, advisories, duplicates
├── crates/
│   ├── trendlib/                   core crate (crates.io: trendlib)
│   │   ├── Cargo.toml              no runtime dependencies
│   │   ├── src/
│   │   │   ├── lib.rs              #![forbid(unsafe_code)]; re-exports; generated convenience fns
│   │   │   ├── core/
│   │   │   │   ├── traits.rs       Stream
│   │   │   │   ├── error.rs        TlError
│   │   │   │   ├── input.rs        Ohlcv views, leading-NaN detection, finiteness checks
│   │   │   │   ├── output.rs       aligned output buffers (NaN / 0 prefill)
│   │   │   │   ├── kernel.rs       Kernel, Step, BarStream: the batch loop and stream
│   │   │   │   └── math.rs         shared kernels (rolling sums, Wilder smoothing, true range)
│   │   │   ├── time/               timestamp + timezone-offset handling for session anchors
│   │   │   ├── registry.rs         GENERATED: static metadata for every indicator
│   │   │   └── indicators/
│   │   │       ├── mod.rs          GENERATED: `pub mod <name>;` for every folder
│   │   │       ├── _enums.yaml     shared enums (MaType, VwapAnchor)
│   │   │       ├── _groups.yaml    group keys, titles, order
│   │   │       ├── rsi/
│   │   │       │   ├── spec.yaml
│   │   │       │   ├── mod.rs      params struct, lookback, batch, stream
│   │   │       │   ├── doc.md
│   │   │       │   └── golden/
│   │   │       │       ├── default.csv
│   │   │       │       └── min_period.csv
│   │   │       └── …               one folder per indicator
│   │   ├── tests/
│   │   │   ├── golden.rs           walks every indicators/*/golden/*.csv
│   │   │   ├── stream_parity.rs    GENERATED: batch == stream for every indicator
│   │   │   └── edge_cases.rs       GENERATED: shared edge-case suite for every indicator
│   │   └── benches/                criterion
│   ├── trendlib-py/                PyO3 extension module `trendlib._core`
│   │   ├── Cargo.toml              pyo3 (abi3-py311), numpy
│   │   └── src/
│   │       ├── lib.rs              module init, error mapping, shared conversion helpers
│   │       └── generated.rs        GENERATED: one #[pyfunction] + one #[pyclass] stream per indicator
│   └── xtask/                      `cargo xtask <cmd>`: generate, regen-check, golden, bench
├── python/trendlib/
│   ├── __init__.py                 public surface; re-exports _functions, stream, registry, errors
│   ├── _functions.py               GENERATED: one wrapper per indicator + uppercase aliases
│   ├── _core.pyi                   GENERATED: stubs for the extension module
│   ├── _convert.py                 NumPy / pandas / polars in-and-out (hand-written)
│   ├── stream.py                   GENERATED factory functions → stream handles
│   ├── registry.py                 list / describe, built from _core metadata
│   ├── errors.py                   TrendLibError, InvalidInput, InsufficientHistory
│   ├── pandas_ext.py               `df.tl.<name>(...)` accessor (registered on import)
│   └── py.typed
├── tests/                          pytest: API contract, conversions, TA-Lib parity, hypothesis
├── scripts/
│   ├── testdata/make_synthetic.py  seeded synthetic OHLCV(+timestamps) datasets
│   └── oracle/talib_golden.py      runs ta-lib-python to produce golden CSVs
├── testdata/                       generated datasets (committed, small)
├── docs/                           this folder + GENERATED docs/indicators/<name>.md pages
├── mkdocs.yml
├── examples/                       notebooks (M5)
├── api/public_api.txt              GENERATED snapshot of the public Python surface
├── .github/workflows/              ci.yml, release.yml, docs.yml
├── .claude/skills/new-indicator/   Claude Code skill
└── README.md, CONTRIBUTING.md, CHANGELOG.md, CODE_OF_CONDUCT.md, SECURITY.md, LICENSE-*
```

## Core crate

### Traits

Settled in M1, generalised in M2. One trait describes an indicator; everything
else is derived from it.

```rust
pub trait Step<const I: usize, const O: usize>: Clone {
    fn push(&mut self, bar: [f64; I]) -> Option<[f64; O]>;   // commit; None while warming up
    fn preview(&self, bar: [f64; I]) -> Option<[f64; O]>;    // what the next push would return
}

pub trait Kernel<const I: usize, const O: usize>: Sized {
    const NAME: &'static str;                       // equals the folder name and spec.yaml `name`
    const INPUTS: [&'static str; I];                // spec.yaml order = Python positional order
    const OUTPUTS: [&'static str; O];               // spec.yaml order = returned tuple order

    type Params: Clone;
    type State: Step<I, O>;

    fn validate(p: &Self::Params) -> Result<(), TlError>;   // ranges from spec.yaml
    fn lookback(p: &Self::Params) -> usize;                 // warm-up bars, see CONVENTIONS.md
    fn state(p: &Self::Params) -> Self::State;

    // Checks beyond "every input is finite", e.g. rejecting a negative volume.
    fn check_inputs(inputs: &[&[f64]; I], from: usize) -> Result<(), TlError> { Ok(()) }

    // Provided: both drive Self::State, so they cannot disagree.
    fn batch(inputs: [&[f64]; I], p: &Self::Params) -> Result<[Vec<f64>; O], TlError>;
    fn open_and_fill(inputs: [&[f64]; I], p: &Self::Params)
        -> Result<(BarStream<Self, I, O>, [Vec<f64>; O]), TlError>;
}

pub trait Stream: Clone + Sized {
    type Params;
    type History<'a>;               // [&'a [f64]; I]: one column per input, as NumPy hands it over
    type Bar: Copy;                 // [f64; I] in spec input order
    type Value: Copy;               // [f64; O] in spec output order

    fn open(history: Self::History<'_>, p: &Self::Params) -> Result<Self, TlError>;
    fn update(&mut self, bar: Self::Bar) -> Result<Self::Value, TlError>;  // commit a closed bar
    fn peek(&self, bar: Self::Bar) -> Result<Self::Value, TlError>;        // evaluate, no commit
    fn value(&self) -> Option<Self::Value>;                                // last committed value
    fn bars_seen(&self) -> u64;                                            // history included
}
```

An indicator writes its parameters, its ranges, its lookback and one `Step`.
`core::kernel` supplies the batch loop, `BarStream`, validation, warm-up
alignment and `open_and_fill`, all of which call that one `Step`. Bitwise parity
between batch and stream is therefore a property of the code shape rather than
something a reviewer has to check.

Two consequences worth stating:

- History arrives **columnar**, one slice per input, because that is what NumPy
  hands over. Opening a stream transposes nothing and allocates nothing per bar.
- Integer outputs travel as `f64` through `Step` and are narrowed at the
  boundary where `spec.yaml` says `dtype: int32`. Every integer an indicator
  produces - pattern flags, trend direction, bar indices - is far inside the
  2^53 a float64 represents exactly, so one step signature covers both.

Implementation rules:

- `batch` and `Stream` share one step function per indicator wherever the
  algorithm allows, so bitwise parity holds by construction rather than by
  coincidence. Batch may add a fast path only if the parity test still passes.
- `Stream::update` and `peek` never allocate. Ring buffers are sized at `open`.
- `peek` must not change any state that a later `update` reads.
- Indicators may call other indicators' kernels (EMA inside MACD); they call the
  kernel functions in `core::math` or the other indicator's module, never Python.
- Errors are values (`TlError`), never panics. A panic in the core is a bug.

### Generated Rust

`cargo xtask generate` writes:

- `indicators/mod.rs` — one `pub mod` per folder, sorted.
- `registry.rs` — `pub static REGISTRY: &[IndicatorMeta]` with every field from
  `spec.yaml` as `&'static str` / numeric literals. No YAML parsing at runtime.
- `lib.rs` convenience section between `// @generated begin` / `// @generated end`
  markers: `pub fn rsi(input, params) -> Result<…>` style free functions.
- `tests/stream_parity.rs` and `tests/edge_cases.rs` — one test fn per
  indicator, calling the shared harness with that indicator's types.

## Python layer

```
user call ── tl.rsi(df["close"], period=14)
   │
   ▼  _functions.py (GENERATED)
   │  validate kwargs → _convert.to_arrays() → call _core.rsi(...) → _convert.wrap(...)
   ▼
 _core.rsi (Rust, generated.rs)
   │  borrow contiguous float64 NumPy arrays (copy only if non-contiguous / other dtype),
   │  release the GIL, run batch, return new NumPy arrays
   ▼
 back in Python: NumPy → same as input type (pandas Series/DataFrame with input index, polars)
```

- `_core` only ever sees NumPy float64 arrays (and int64 epoch-ns for timestamps).
  All pandas/polars handling lives in `_convert.py`, so the Rust side stays small.
- Errors map: `TlError::InvalidParam`/`InvalidInput` → `trendlib.InvalidInput`
  (subclass of `ValueError`), `TlError::InsufficientHistory` →
  `trendlib.InsufficientHistory`. All subclass `trendlib.TrendLibError`.
- Docstrings for wrappers are generated from `spec.yaml` + the first paragraph of
  `doc.md` (NumPy docstring style), so IDE help matches the docs site.
- The uppercase aliases (`RSI`, `BBANDS`) are thin generated wrappers that rename
  parameters and outputs per `spec.yaml` `talib:` fields (D12).

## Code generator (`crates/xtask`)

Inputs: every `indicators/*/spec.yaml`, `_enums.yaml`, `_groups.yaml`, the first
paragraph and Formula section of each `doc.md`.

Outputs (all carry the `@generated` banner): `indicators/mod.rs`, `registry.rs`,
the generated section of `lib.rs`, `trendlib-py/src/generated.rs`,
`python/trendlib/_functions.py`, `python/trendlib/stream.py`,
`python/trendlib/_core.pyi`, `tests/stream_parity.rs`, `tests/edge_cases.rs`,
`docs/indicators/<name>.md`, `docs/indicators/index.md`, `api/public_api.txt`.

Rules:

- Deterministic: sorted iteration, no timestamps in output, `\n` line endings.
  Running it twice changes nothing.
- Validates specs strictly (unknown keys, bad ranges, defaults outside range,
  duplicate names, input/output name clash) and fails with file:line messages.
- Generated Rust is passed through `rustfmt`; generated Python through
  `ruff format` if available (CI has it).
- Use a maintained YAML crate (`serde_yaml` is unmaintained).

`regen-check` = run `generate`, then fail if `git status --porcelain` is non-empty.
CI runs it on every PR.

## Versioning

One version for the workspace (`[workspace.package] version`), inherited by every
crate; maturin reads it for the wheel (`dynamic = ["version"]`). See `RELEASE.md`.
