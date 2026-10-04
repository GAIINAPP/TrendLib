# Prior art: TA-Lib

TA-Lib ([github.com/TA-Lib/ta-lib](https://github.com/TA-Lib/ta-lib)) is the
reference library for technical indicators and TrendLib's oracle for shared
indicators. This page records how it is built and what TrendLib copies or changes.
Facts are from the repository at version 0.8.1 (released 2026-09-12) and
ta-lib-python 0.8.1, checked 2026-10-04.

## How TA-Lib is built

- **201 functions** in 10 groups; 61 are candlestick patterns. Recent additions
  include SUPERTREND, VWAP (cumulative, never reset), KC, DONCHIAN, HMA, RMA.
- **One folder per function** under `ta_codegen/input/<name>/` with three files:
  `<name>.yaml` (metadata only), `<name>.c` (a `_lookback()` and a batch function in
  plain C), `<name>.md` (summary, formula, references).
- **A Rust code generator** (`ta_codegen/generator`) parses those inputs into an IR
  and renders the C library, native Rust/Java/C# ports, a streaming API, JSON-RPC
  test servers, benchmarks and the website's function pages. Generated files are
  never hand-edited.
- **PR gate `regen-check`:** regenerate everything; any `git status` change fails
  the PR.
- **Correctness:** hardcoded golden values from an independent oracle or a
  published table, plus a regression harness that cross-checks every language
  against the C reference, bitwise in the strictest gates. Their contributor guide
  explicitly forbids "golden" values computed by re-deriving the formula.
- **Streaming API** (`Open / Update / Peek / Close / Clone`), O(1) per bar,
  bit-identical to batch.
- **Calling convention (C):** `startIdx, endIdx`, input arrays, optional params,
  `outBegIdx, outNBElement`, output arrays; callers size buffers with
  `TA_<NAME>_Lookback()`. ta-lib-python hides this and returns NaN-padded arrays
  aligned to the input.
- **Global settings:** unstable period and candle settings are process-wide.
- **Agent-ready:** a `CLAUDE.md` and Claude Code skills (`new-ta-func` and others);
  their stated model is human-written specs, AI-written code, human-reviewed oracles.
- **Distribution:** ta-lib-python 0.8.1 ships binary wheels for Linux (glibc and
  musl), macOS and Windows, so no separate C install is needed; wrappers exist for
  R, Go, Ruby, PHP, Zig, PostgreSQL and more.

## What TrendLib copies

| TA-Lib practice | TrendLib form |
| --- | --- |
| One folder per indicator, metadata separate from logic | `indicators/<name>/` with `spec.yaml`, `mod.rs`, `doc.md`, `golden/` |
| Generated metadata, bindings and docs | `cargo xtask generate` |
| `regen-check` PR gate | Same name, same rule |
| Golden values from an executed independent oracle | `TESTING.md` § 2; TA-Lib itself is the oracle for shared indicators |
| Streaming API bit-identical to batch | `tl.stream.*`, bitwise parity tests |
| Lookback as a first-class concept | `tl.lookback()`, registry |
| Computation conventions and defaults | D6: identical for shared indicators, so parity is testable |
| Machine-readable function catalogue | `tl.registry.to_json()` |
| Agent instructions in the repo | `CLAUDE.md`, `.claude/skills/new-indicator` |

## What TrendLib changes

| TA-Lib | TrendLib | Why |
| --- | --- | --- |
| C core, ports generated into 4 languages | One Rust core; Python via PyO3 | One implementation to verify |
| Global unstable-period / candle settings | None (D9) | Thread safety, reproducibility |
| NaN mid-series undefined | Raises (D8) | No silent wrong values |
| NumPy only (Python) | NumPy, pandas, polars in and out; index preserved | How Python users hold data |
| VWAP never resets; no timestamps anywhere | Session-anchored VWAP with timezone-aware timestamps | Intraday use in Indian markets |
| No CPR / pivot levels | `cpr`, `pivots_traditional`, `pivots_camarilla` | Widely used by Indian traders |
| Metadata via abstract API | Registry with plot hints + JSON export | Indicator pickers, LLM tool schemas |
| Parameter names like `timeperiod` | `period` etc., with TA-Lib aliases (D12) | Readability, easy migration |

## Honest positioning

TA-Lib already offers speed, breadth (201 functions), wheels and Supertrend. TrendLib
competes on ergonomics (DataFrames, aligned outputs, clear errors), streaming parity
in Python, session awareness and Indian-market levels, metadata for applications,
and a Rust crate. Claims in the README must stay within what tests demonstrate.
