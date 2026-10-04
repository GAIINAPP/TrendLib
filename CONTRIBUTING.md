# Contributing to TrendLib

Thanks for helping. TrendLib values correctness over breadth: every number we ship is
backed by an independent reference, so contributions follow a spec-first process.

## Development setup

Requirements: Rust (via rustup; the repo pins the toolchain), Python ≥ 3.11.

```bash
git clone https://github.com/<org>/trendlib && cd trendlib
python -m venv .venv && source .venv/bin/activate      # Windows: .venv\Scripts\activate
pip install maturin
pip install -e ".[dev]"            # builds the extension in debug mode
cargo test --workspace && pytest
```

Use `maturin develop --release` when benchmarking.

## Adding an indicator

1. **Open a "New indicator spec" issue** (template provided): name, group, inputs,
   parameters with defaults and ranges, outputs, formula, a citable reference, and
   which oracle can verify it.
2. **Wait for a maintainer to approve the spec.** Defaults and ranges are never
   chosen during implementation.
3. **Copy the closest folder** in `crates/trendlib/src/indicators/` and write
   `spec.yaml`, `mod.rs`, `doc.md` (formats: `docs/SPEC_FORMAT.md`).
4. **Produce golden data by running the oracle**: `cargo xtask golden <name>` for
   TA-Lib-backed indicators, or transcribed reference values (`docs/TESTING.md` § 2).
   Never compute expected values with your own implementation.
5. **Generate and test:** `cargo xtask generate`, `cargo test --workspace`,
   `maturin develop && pytest`, `cargo xtask regen-check`.
6. **Show the test can fail:** break the implementation once, confirm the golden test
   fails, restore it, and say so in the PR.
7. **Open a PR** linking the spec issue. The diff should touch only your indicator's
   folder and the generated files it owns.

## Other changes

- Bugs: open an issue with a minimal reproduction (input, params, expected vs actual,
  and where the expected value comes from).
- Docs and tooling PRs are welcome without an issue if small.
- Changes to shared test harnesses, tolerances or conventions need maintainer
  agreement first.

## Pull request rules

- Branch from `main`; keep PRs focused.
- Every commit carries a DCO sign-off (`git commit -s`), certifying you have the right
  to submit it under the project's licenses (MIT OR Apache-2.0).
- CI must be green: format, clippy, tests on three OSes, `regen-check`, API snapshot.
- User-visible changes add a line to `CHANGELOG.md` under "Unreleased".

## Naming and wording

Indicators describe markets; they do not advise. Do not name functions, outputs or
parameters as trade instructions (`buy`, `sell`, `entry`, `exit`, `target`,
`recommendation`), and keep docs descriptive. Established indicator terms such as
MACD's signal line are fine.

## AI-assisted contributions

Welcome. `CLAUDE.md` and `.claude/skills/new-indicator` give agents the same rules as
humans. The human submitting the PR is responsible for it, including the oracle's
provenance.

## Code of conduct

See `CODE_OF_CONDUCT.md`.
