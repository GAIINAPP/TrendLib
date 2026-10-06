"""Time every shared indicator against TA-Lib and write the table M4 asks for.

    python scripts/bench/compare.py                  # 1,000,000 bars
    python scripts/bench/compare.py --bars 100000    # quicker, same shape
    python scripts/bench/compare.py --only rsi,macd

`docs/TESTING.md` section 8: both libraries run on the same NumPy arrays and
the median of several runs is reported. The nightly job posts the table; the
release checklist passes `--enforce`, which exits non-zero when a shared
indicator is over budget without a reason recorded in `SLOWER_ON_PURPOSE`.
"""

from __future__ import annotations

import argparse
import functools
import json
import statistics
import time
from pathlib import Path

import numpy as np

REPO_ROOT = Path(__file__).resolve().parents[2]
INDICATORS = REPO_ROOT / "crates/trendlib/src/indicators"
ENUMS = INDICATORS / "_enums.yaml"

# The ratio the release checklist allows.
BUDGET = 1.5

# Why an indicator is allowed past the budget. The reason has to be measured,
# not guessed, and `docs/TESTING.md` section 8 repeats it.
RESUMMED = (
    "walks its window every bar instead of carrying a running total, which is "
    "what makes it more accurate than the oracle (CONVENTIONS.md section 1)"
)
SLOWER_ON_PURPOSE: dict[str, str] = {}

# Where TrendLib's default is not what TA-Lib computes, the comparison runs the
# setting that matches it, or it would be timing two different jobs.
MATCHES_ORACLE_WITH = {"vwap": {"anchor": "none"}}


def build_columns(bars):
    """Every column any indicator can ask for, built once and shared.

    Rebuilding them per indicator would spend more time allocating a hundred
    megabytes than running the indicator.
    """
    rng = np.random.default_rng(20261006)
    base = 1000.0 * np.cumprod(1.0 + rng.normal(0.0, 0.01, bars))
    return {
        "close": base,
        "source": base,
        "source0": base,
        "open": base * 0.75 + 1.0,
        "source1": base * 0.75 + 1.0,
        "high": base + np.abs(base) * 0.005 + 0.5,
        "low": base - np.abs(base) * 0.005 - 0.5,
        "volume": np.abs(base) * 10.0,
        "periods": 2.0 + (np.arange(bars) % 29).astype(np.float64),
        "timestamps": 1_767_225_600_000_000_000 + np.arange(bars) * 86_400_000_000_000.0,
    }


def median_ms(call, repeats):
    timings = []
    for _ in range(repeats):
        start = time.perf_counter_ns()
        call()
        timings.append((time.perf_counter_ns() - start) / 1e6)
    return statistics.median(timings)


def ma_type_ints(yaml):
    return yaml.safe_load(ENUMS.read_text(encoding="utf-8"))["MaType"]["talib_int"]


def shared(yaml):
    """Every indicator TA-Lib also has, with its alias block."""
    import trendlib as tl

    found = []
    for name in sorted(tl._core.INPUTS):
        spec = yaml.safe_load((INDICATORS / name / "spec.yaml").read_text(encoding="utf-8"))
        alias = spec.get("talib")
        if alias:
            found.append((name, alias))
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bars", type=int, default=1_000_000)
    parser.add_argument("--repeats", type=int, default=5)
    parser.add_argument("--only", default="", help="comma-separated names or substrings")
    parser.add_argument("--json", type=Path, help="also write the rows here")
    parser.add_argument(
        "--enforce",
        action="store_true",
        help="exit non-zero when an indicator is over budget with no recorded reason",
    )
    args = parser.parse_args()

    import talib
    import yaml

    import trendlib as tl
    from trendlib import _core

    wanted = [part for part in args.only.split(",") if part]
    numbers = ma_type_ints(yaml)
    built = build_columns(args.bars)
    rows = []
    for name, alias in shared(yaml):
        if wanted and not any(part in name for part in wanted):
            continue
        kinds = list(_core.KINDS[name])
        bars = [built["close" if kind == "series" else kind] for kind in kinds]
        params = {
            param: spec["default"] for param, spec in _core.PARAMS[name].items()
        } | MATCHES_ORACLE_WITH.get(name, {})
        renames = alias.get("params") or {}
        oracle_params = {
            renames[key]: numbers[value] if isinstance(value, str) else value
            for key, value in params.items()
            if key in renames
        }
        # TA-Lib has no timestamps input: it expects the caller to have sliced
        # the sessions already.
        oracle_bars = [
            column for column, kind in zip(bars, kinds, strict=True) if kind != "timestamps"
        ]
        ours = getattr(tl, name)
        theirs = getattr(talib, alias["name"])
        mine = median_ms(functools.partial(ours, *bars, **params), args.repeats)
        oracle = median_ms(functools.partial(theirs, *oracle_bars, **oracle_params), args.repeats)
        rows.append(
            {
                "indicator": name,
                "trendlib_ms": mine,
                "talib_ms": oracle,
                "ratio": mine / oracle if oracle else float("inf"),
            }
        )

    rows.sort(key=lambda row: row["ratio"], reverse=True)
    over = [
        row for row in rows if row["ratio"] > BUDGET and row["indicator"] not in SLOWER_ON_PURPOSE
    ]

    print(f"# Benchmarks: {args.bars:,} bars, median of {args.repeats} runs\n")
    print("| Indicator | TrendLib (ms) | TA-Lib (ms) | Ratio |")
    print("| --- | ---: | ---: | ---: |")
    for row in rows:
        flag = (
            ""
            if row["ratio"] <= BUDGET
            else (" (slower on purpose)" if row["indicator"] in SLOWER_ON_PURPOSE else " OVER")
        )
        print(
            f"| `{row['indicator']}` | {row['trendlib_ms']:.2f} | "
            f"{row['talib_ms']:.2f} | {row['ratio']:.2f}x{flag} |"
        )
    if SLOWER_ON_PURPOSE:
        print("\nSlower on purpose:\n")
        for name, reason in sorted(SLOWER_ON_PURPOSE.items()):
            print(f"- `{name}`: {reason}")
    print(
        f"\n{len(rows)} shared indicators, "
        f"{sum(1 for row in rows if row['ratio'] <= BUDGET)} within {BUDGET}x TA-Lib."
    )

    if args.json:
        args.json.write_text(json.dumps(rows, indent=2), encoding="utf-8")

    if over:
        print(
            f"\n{len(over)} over budget with no recorded reason, worst first: "
            + ", ".join(f"{row['indicator']} ({row['ratio']:.1f}x)" for row in over[:10])
            + ("..." if len(over) > 10 else "")
        )
    return 1 if (over and args.enforce) else 0


if __name__ == "__main__":
    raise SystemExit(main())
