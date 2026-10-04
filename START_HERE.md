# Start here (for you, not for Claude Code)

> **Internal, not for the public repo.** This file describes how the project is
> driven, milestone by milestone. Delete it before the repository goes public,
> or keep it out of the first public commit. Nothing else in `docs/` is internal.

This pack is the starting repo for TrendLib. Claude Code reads `CLAUDE.md`
automatically and follows the docs in `docs/`. You approve each milestone.

## 1. One-time setup (≈ 20 minutes)

1. Create an empty GitHub repo named `trendlib` (decide Q1 in `docs/DECISIONS.md`:
   GAIIN org or a neutral org).
2. Unzip this pack into the repo, commit and push:
   ```bash
   git init && git add . && git commit -s -m "Add TrendLib spec and contributor docs"
   git branch -M main && git remote add origin git@github.com:<org>/trendlib.git && git push -u origin main
   ```
3. Install Rust (`https://rustup.rs`) and Python 3.11+ on the machine where Claude
   Code runs.
4. Open Claude Code in the repo folder.

## 2. Prompts, one milestone at a time

Paste one prompt, let it finish, read its report, answer its questions, review the
branch, then merge before the next prompt.

**M0**

```
Read CLAUDE.md and every file it lists under "Read first". Then build milestone M0
from docs/MILESTONES.md, and only M0.

Before writing code, list anything in the docs that is ambiguous, contradictory or
that you believe is wrong, with your proposed resolution. Then implement M0 on branch
m0-scaffold in small commits, run every M0 acceptance check, and stop with the
milestone report from CLAUDE.md. Do not start M1.
```

**M1 … M5** (change the number and slug)

```
Build milestone M1 from docs/MILESTONES.md on branch m1-core. Same rules as before:
re-read CLAUDE.md, flag ambiguities first, run every acceptance check, stop with the
milestone report. Do not start the next milestone.
```

Suggested slugs: `m1-core`, `m2-codegen`, `m3-packaging`, `m4-indicators`,
`m5-release`. M4 is large; you can ask for it in parts:
`Build M4 steps 1–4 of the build order in docs/INDICATORS.md § 4 using the
new-indicator skill.`

**When an indicator needs a human oracle (CPR, pivots)**

Collect 10–20 rows of reference values (e.g. from TradingView's "Pivot Points
Standard" on daily NIFTY bars: date, prior-day OHLC, each level), then:

```
Here are reference values for pivots_traditional from TradingView (Pivot Points
Standard, type Traditional, NSE:NIFTY daily, transcribed by <name> on <date>):
<paste table>. Write them as golden/default.csv per docs/SPEC_FORMAT.md § 4 with
produced_by: manual, enable the test, and report the result.
```

## 3. Things only you can do

| When | What | Where |
| --- | --- | --- |
| Before M3 | Answer Q1 (GitHub org) | `docs/DECISIONS.md` |
| Before M3 | PyPI + TestPyPI accounts, trusted publishers, GitHub environments | `docs/RELEASE.md` § 4 |
| After M3 | Approve the first `0.1.0a1` PyPI publish (secures the name) | GitHub Actions → Deployments |
| During M4 | Provide CPR / pivot reference values (Q4) | prompt above |
| Before M5 | Docs domain (Q5); legal glance at license and README | `docs/DECISIONS.md` |
| Any time | Trademark "TrendLib" if you want brand protection | outside the repo |

## 4. What is in this pack

| File | For |
| --- | --- |
| `CLAUDE.md` | Rules Claude Code follows in every session |
| `docs/MILESTONES.md` | The build plan with acceptance checks |
| `docs/DECISIONS.md` | Settled decisions and open questions |
| `docs/SPEC.md` | Product requirements |
| `docs/ARCHITECTURE.md` | Repo layout, traits, generator, Python layer |
| `docs/PYTHON_API.md` | The public API contract |
| `docs/CONVENTIONS.md` | Numerical rules and deviations from TA-Lib |
| `docs/INDICATORS.md` | The approved indicator list with defaults and formulas |
| `docs/SPEC_FORMAT.md` | File formats inside each indicator folder |
| `docs/TESTING.md` | Oracles, golden data, test suites, CI |
| `docs/RELEASE.md` | pyproject, wheels, PyPI publishing |
| `docs/PRIOR_ART_TALIB.md` | What we learned from TA-Lib |
| `README.md`, `CONTRIBUTING.md` | Public-facing docs (Claude Code keeps them true) |
| `.claude/skills/new-indicator/` | Claude Code skill for adding indicators |
| `.github/` | Issue template for indicator specs, PR template |

You can delete this file once the repo is running.
