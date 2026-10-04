## What this changes

<!-- One or two sentences. Link the spec issue for a new indicator. -->

## Checklist

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `cargo xtask regen-check` (from M2)
- [ ] `maturin develop --release && pytest`
- [ ] `CHANGELOG.md` updated under "Unreleased" if this is user-visible
- [ ] Every commit is signed off (`git commit -s`, DCO)

## For a new or changed indicator

- [ ] Spec issue approved before implementation; defaults and ranges come from it
      or from `docs/INDICATORS.md`, not from this PR
- [ ] `spec.yaml`, `mod.rs`, `doc.md` and `golden/*.csv` all present
- [ ] Golden values were **produced by running an oracle**, not by this
      implementation or a re-derivation of its formula

  Oracle: <!-- e.g. ta-lib-python 0.8.1 via scripts/oracle/talib_golden.py, or a
  transcriber + source + date for a human oracle -->

- [ ] I broke the implementation once, confirmed the golden test went red, and
      restored it

  How it was broken: <!-- e.g. window start off by one in the rolling sum -->

- [ ] No tolerance was loosened and no shared test was edited to make this pass
- [ ] Names and docs are descriptive, not advisory (`docs/DECISIONS.md` D11)

## Notes for the reviewer

<!-- Anything surprising: a deviation added to CONVENTIONS.md section 9, a doc
that turned out wrong and was fixed here, an assumption recorded in
docs/DECISIONS.md. -->
