"""Produce chart-pattern golden CSVs by running ta-patterns, the P oracle.

    python scripts/oracle/chart_golden.py chart_double_top                 # every case
    python scripts/oracle/chart_golden.py chart_double_top --case default  # one case

The chart patterns (``docs/INDICATORS.md`` section 5.1) have no TA-Lib
equivalent, so their expected values come from ``ta-patterns`` instead
(``docs/TESTING.md`` section 2). This script calls the oracle function the
section 5.1 table names for each TrendLib function and writes what it returns,
times 100, unchanged otherwise. It never computes an expected value itself.

Two TrendLib functions are the two directions of one shape, which the oracle
splits into two functions; for those the upward reading is taken where it fired
and the downward one elsewhere, the order TrendLib tests them in.

Rows a ``docs/CONVENTIONS.md`` section 9 deviation touches are excluded and
named in the header. They are found with the oracle's own swing-point functions,
never with TrendLib.
"""

from __future__ import annotations

import argparse
import csv
import datetime as dt
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
INDICATORS = REPO_ROOT / "crates" / "trendlib" / "src" / "indicators"
APPROVED = REPO_ROOT / "docs" / "INDICATORS.md"
TESTDATA = REPO_ROOT / "testdata"

TOLERANCE = "rel=1e-10 abs=1e-12"
DAILY = "daily_2000.csv"
CHARTS = "charts_2579.csv"
PATTERNS = "patterns_1080.csv"
SHAPES = "shapes_528.csv"
INTRADAY = "intraday_5m_20d.csv"

# TrendLib's names for the oracle's parameters (D12): only `window` differs.
RENAMED = {"period": "window"}

# The oracle functions that draw their lines with the sliding least-squares fit,
# which is where deviation 9 can apply. Every other function draws none.
TRENDLINE = {
    "ascending_triangle",
    "descending_triangle",
    "symmetrical_triangle",
    "broadening_top",
    "broadening_bottom",
    "rising_wedge",
    "falling_wedge",
    "rectangle_top",
    "rectangle_bottom",
    "channel_asc",
    "channel_desc",
    "flag_bull",
    "flag_bear",
    "pennant_bull",
    "pennant_bear",
    "broadening_wedge_asc",
    "broadening_wedge_desc",
    "right_angle_broadening_asc",
    "right_angle_broadening_desc",
}
POLE = {"flag_bull", "flag_bear", "pennant_bull", "pennant_bear"}

ROW = re.compile(r"^\| `((?:chart|bar|harmonic)_[a-z0-9_]+)` \| [^|]+ \| ([^|]+) \|")

# Boundaries the default and minimum cases cannot reach, each pinned with a
# setting at which the daily walk reaches it.
#
# A repeated top within `pivot_n` bars counts once, so two tops are always more
# than `pivot_n` apart; while `min_separation` is no larger than `pivot_n`, as
# it is at both the defaults and the minimums, the separation test can never
# bind. These values were found by breaking that comparison by one bar and
# keeping a value where the output changed.
#
# A head and shoulders neckline waits `period` bars; at 150 no close on the
# daily walk ever crosses one on its last bar. These short windows were found
# the same way, by moving the wait by one bar.
# Chart shapes that fire in one direction on no walk: the islands' gaps and the
# bump-and-run bottom occur among the candle shapes, and the high and tight
# flag's 40 percent pole only where one was built.
ON_PATTERNS = {
    "chart_island_top",
    "chart_island_bottom",
    "chart_bump_and_run_top",
    "chart_bump_and_run_bottom",
}
ON_SHAPES = {"chart_high_tight_flag", "chart_three_peaks", "chart_three_valleys"}
# Two rounded tops a few percent apart are what five-minute bars draw and a daily
# walk does not.
ON_INTRADAY = {"chart_double_top_eve_eve"}

EXTRA_CASES = {
    "chart_double_top": {"wide_separation": {"min_separation": 10}},
    "chart_double_bottom": {"wide_separation": {"min_separation": 13}},
    "chart_triple_top": {"wide_separation": {"min_separation": 9}},
    "chart_triple_bottom": {"wide_separation": {"min_separation": 6}},
    "chart_head_shoulders": {"short_neckline": {"period": 20, "pivot_n": 1, "min_separation": 2}},
    "chart_inverse_head_shoulders": {
        "short_neckline": {"period": 15, "pivot_n": 3, "min_separation": 3}
    },
    # An island search one bar deeper, and V arms split the other way, change
    # nothing until the nearest gap sits at the far end of a short search and
    # until the period is odd; these reach both, on the bars dataset_for names.
    "chart_island_top": {"near_gap": {"max_island_bars": 2}},
    "chart_island_bottom": {"near_gap": {"max_island_bars": 2}},
    "chart_v_bottom": {"odd_period": {"period": 15}},
    "chart_v_top": {"odd_period": {"period": 15}},
}


def oracle_functions(name: str) -> list[str]:
    """The oracle functions sections 5.1 and 5.2 name for `name`, upward one first."""
    section = APPROVED.read_text(encoding="utf-8").split("### 5.1 Chart patterns", 1)[1]
    for line in section.splitlines():
        row = ROW.match(line)
        if row and row.group(1) == name:
            return re.findall(r"`([a-z0-9_]+)`", row.group(2))
    raise SystemExit(f"{name} is not in docs/INDICATORS.md section 5.1 or 5.2")


def load_spec(name: str) -> dict:
    import yaml

    path = INDICATORS / name / "spec.yaml"
    if not path.exists():
        raise SystemExit(f"no spec at {path.relative_to(REPO_ROOT)}")
    spec = yaml.safe_load(path.read_text(encoding="utf-8"))
    if "chart_pattern" not in (spec.get("flags") or []):
        raise SystemExit(
            f"{name} is not a chart pattern; its oracle is scripts/oracle/talib_golden.py"
        )
    return spec


def defaults(spec: dict) -> dict:
    return {param["name"]: param["default"] for param in spec.get("params") or []}


def cases(spec: dict) -> dict[str, dict]:
    """`default` and `min_period` read the daily walk; `charts` reads the shapes.

    `min_period` pins every whole-numbered parameter to its documented minimum,
    the same meaning the case has for every other indicator. `charts` reads
    bars built so the pattern fires in each direction it reads
    (`docs/TESTING.md` section 3).
    """
    floors = {
        param["name"]: param["min"]
        for param in spec.get("params") or []
        if param["type"] == "int" and "min" in param
    }
    found = {
        "default": defaults(spec),
        "min_period": {**defaults(spec), **floors},
        "charts": defaults(spec),
    }
    # A bar pattern reads a few bars at a time, like a candle, so it also gets
    # the bars built for the candlestick patterns, where the rarer shapes occur;
    # so do the few chart shapes that fire only there or on the shapes dataset.
    if spec["group"] == "bars" or spec["name"] in ON_PATTERNS:
        found["patterns"] = defaults(spec)
    if spec["name"] in ON_SHAPES:
        found["shapes"] = defaults(spec)
    if spec["name"] in ON_INTRADAY:
        found["intraday"] = defaults(spec)
    for case, overrides in EXTRA_CASES.get(spec["name"], {}).items():
        found[case] = {**defaults(spec), **overrides}
    return found


def dataset_for(case: str) -> str:
    return {
        "charts": CHARTS,
        "patterns": PATTERNS,
        "shapes": SHAPES,
        "intraday": INTRADAY,
        "near_gap": PATTERNS,
        "odd_period": CHARTS,
    }.get(case, DAILY)


def read_columns(filename: str):
    import numpy as np

    with (TESTDATA / filename).open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    return {
        key: np.array([float(row[key]) for row in rows]) for key in ("open", "high", "low", "close")
    }


def takes_mode(detector) -> bool:
    """Whether an oracle function, or the halves it combines, take `mode`."""
    import inspect

    return "mode" in inspect.signature(detector).parameters


def run_oracle(name: str, columns: dict, params: dict):
    import numpy as np
    import ta_patterns.chart_patterns as cp

    keywords = {RENAMED.get(key, key): value for key, value in params.items()}
    bars = (columns["open"], columns["high"], columns["low"], columns["close"])
    readings = []
    for function in oracle_functions(name):
        detector = getattr(cp, function)
        # The bar patterns take no `mode`: they are complete on the bar they
        # read. Everything else is asked for its confirmed reading.
        confirmed = {"mode": "confirmed"} if takes_mode(detector) else {}
        readings.append(detector(*bars, **confirmed, **keywords).astype(np.int64) * 100)
    if len(readings) == 1:
        return readings[0]
    upward, downward = readings
    return np.where(upward != 0, upward, downward)


def flat_rows(columns: dict, params: dict, first: int) -> list[int]:
    """Rows whose swing highs or swing lows in the window all share one price.

    Deviation 9: the oracle's slope for such a line is a residue of running
    totals, TrendLib's is exactly 0. Swing points come from the oracle.
    """
    import numpy as np
    from ta_patterns.chart_patterns._core import pivot_highs, pivot_info, pivot_lows

    n, period = params["pivot_n"], params["period"]
    sides = []
    for prices, finder in ((columns["high"], pivot_highs), (columns["low"], pivot_lows)):
        at, price = pivot_info(prices, finder(prices, n), n)
        sides.append((at, price))
    rows = []
    for row in range(first, len(columns["close"])):
        for at, price in sides:
            inside = price[(at >= row - period) & (at <= row)]
            if inside.size >= 2 and np.all(inside == inside[0]):
                rows.append(row)
                break
    return rows


def ranges(rows: list[int]) -> list[tuple[int, int]]:
    spans: list[tuple[int, int]] = []
    for row in rows:
        if spans and row == spans[-1][1] + 1:
            spans[-1] = (spans[-1][0], row)
        else:
            spans.append((row, row))
    return spans


def excluded_rows(name: str, columns: dict, params: dict) -> str:
    functions = set(oracle_functions(name))
    found: list[str] = []
    if functions & POLE:
        row = params["pole_bars"] + params["period"]
        if row < len(columns["close"]):
            found.append(f"{row}-{row} (Deviation 8)")
    if functions & TRENDLINE:
        first = params["period"]
        if functions & POLE:
            first = params["pole_bars"] + params["period"] + 1
        found += [f"{a}-{b} (Deviation 9)" for a, b in ranges(flat_rows(columns, params, first))]
    return ", ".join(found) or "none"


def write_case(name: str, case: str, spec: dict, params: dict) -> Path:
    import ta_patterns

    dataset = dataset_for(case)
    columns = read_columns(dataset)
    readings = run_oracle(name, columns, params)

    functions = oracle_functions(name)
    import ta_patterns.chart_patterns as cp

    mode = ", mode=confirmed" if takes_mode(getattr(cp, functions[0])) else ""
    named = " and ".join(f"ta_patterns.chart_patterns.{f}" for f in functions)
    how = (
        f"{named}, the first where it fires and the second elsewhere"
        if len(functions) == 2
        else named
    )
    rendered = ", ".join(f"{key}={value}" for key, value in params.items()) or "none"
    header = [
        f"# indicator: {name}",
        f"# case: {case}",
        f"# params: {rendered}",
        f"# oracle: ta-patterns {ta_patterns.__version__}, {how}{mode}, times 100",
        f"# produced_by: python scripts/oracle/chart_golden.py {name} --case {case}",
        f"# input: testdata/{dataset} (all rows)",
        f"# tolerance: {TOLERANCE}",
        f"# excluded_rows: {excluded_rows(name, columns, params)}",
        f"# date: {dt.date.today().isoformat()}",
    ]

    names = [i["name"] for i in spec["inputs"]] + [o["name"] for o in spec["outputs"]]
    lines = [*header, ",".join(names)]
    for row in range(len(readings)):
        cells = [repr(float(columns[i["name"]][row])) for i in spec["inputs"]]
        cells.append(str(int(readings[row])))
        lines.append(",".join(cells))

    path = INDICATORS / name / "golden" / f"{case}.csv"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    size = path.stat().st_size
    if size > 2_000_000:
        raise SystemExit(
            f"{path.name} is {size} bytes; docs/SPEC_FORMAT.md section 4 caps it at 2 MB"
        )
    return path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("indicator", help="folder name under crates/trendlib/src/indicators")
    parser.add_argument("--case", help="one case; default is every case")
    args = parser.parse_args()

    try:
        import ta_patterns  # noqa: F401
        import yaml  # noqa: F401
    except ImportError as exc:
        raise SystemExit(
            f"{exc.name} is missing; install the dev extra: pip install -e '.[dev]'"
        ) from exc

    spec = load_spec(args.indicator)
    available = cases(spec)
    if args.case and args.case not in available:
        raise SystemExit(f"unknown case {args.case!r}; known cases: {', '.join(available)}")
    wanted = {args.case: available[args.case]} if args.case else available
    for case, params in wanted.items():
        path = write_case(args.indicator, case, spec, params)
        print(f"wrote {path.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
