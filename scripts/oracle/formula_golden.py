"""Produce golden CSVs for the oracle-A indicators of INDICATORS.md section 5.3.

    python scripts/oracle/formula_golden.py ichimoku                 # every case
    python scripts/oracle/formula_golden.py ichimoku --case default  # one case

No runnable library implements these, so the owner chose oracle A for them
(``docs/DECISIONS.md`` D20, ``docs/TESTING.md`` section 2): the published
formula the section 5.3 row cites, evaluated through TA-Lib's own functions
wherever one computes a step and NumPy for the rest. Every number a TA-Lib
function can produce comes from TA-Lib; only the shape of the expression is
written here, and each file's header names the functions that produced it.

This is the weaker oracle. It proves TrendLib computes the documented formula,
not that the formula is the one a given platform draws.
"""

from __future__ import annotations

import argparse
import csv
import datetime as dt
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
INDICATORS = REPO_ROOT / "crates" / "trendlib" / "src" / "indicators"
TESTDATA = REPO_ROOT / "testdata"

TOLERANCE = "rel=1e-10 abs=1e-12"
DAILY = "daily_2000.csv"


def load_spec(name: str) -> dict:
    import yaml

    path = INDICATORS / name / "spec.yaml"
    if not path.exists():
        raise SystemExit(f"no spec at {path.relative_to(REPO_ROOT)}")
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def defaults(spec: dict) -> dict:
    return {param["name"]: param["default"] for param in spec.get("params") or []}


def cases(spec: dict) -> dict[str, dict]:
    """`default`, and `min_period` with every whole-numbered parameter at its minimum."""
    found = {"default": defaults(spec)}
    floors = {
        param["name"]: param["min"]
        for param in spec.get("params") or []
        if param["type"] == "int" and "min" in param
    }
    if floors:
        found["min_period"] = {**defaults(spec), **floors}
    return found


def read_columns(filename: str):
    import numpy as np

    with (TESTDATA / filename).open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    columns = {
        key: np.array([float(row[key]) for row in rows])
        for key in ("open", "high", "low", "close", "volume")
    }
    columns["source"] = columns["close"]
    return columns


def shifted(values, bars: int):
    """`values` drawn `bars` rows later: row t holds what row t - bars computed."""
    import numpy as np

    if bars == 0:
        return values.copy()
    out = np.full_like(values, np.nan)
    out[bars:] = values[:-bars]
    return out


def midpoint(high, low, period: int):
    """TA-Lib's MIDPRICE, or its MEDPRICE for the one-bar window MIDPRICE refuses."""
    import talib

    return talib.MIDPRICE(high, low, timeperiod=period) if period >= 2 else talib.MEDPRICE(high, low)


# Each formula returns its outputs in spec order, with the TA-Lib functions it
# used for the header. Rows before the indicator's warm-up are blanked by
# `warm_up`, which is TrendLib's alignment rule, not part of the formula.
def ichimoku(cols, p):
    import numpy as np
    import talib

    tenkan = midpoint(cols["high"], cols["low"], p["tenkan_period"])
    kijun = midpoint(cols["high"], cols["low"], p["kijun_period"])
    senkou = midpoint(cols["high"], cols["low"], p["senkou_period"])
    span_a = talib.MULT(talib.ADD(tenkan, kijun), np.full_like(tenkan, 0.5))
    d = p["displacement"]
    longest = max(p["tenkan_period"], p["kijun_period"], p["senkou_period"])
    outputs = [tenkan, kijun, shifted(span_a, d), shifted(senkou, d)]
    return outputs, longest - 1 + d, "talib.MIDPRICE (MEDPRICE for a one-bar window), ADD and MULT"


FORMULAS = {
    "ichimoku": ichimoku,
}


def warm_up(outputs, lookback: int):
    """Every output is warm-up before `lookback`, as TrendLib aligns a multi-output row."""
    import numpy as np

    blanked = []
    for values in outputs:
        values = np.array(values, dtype=np.float64)
        values[:lookback] = np.nan
        blanked.append(values)
    return blanked


def write_case(name: str, case: str, spec: dict, params: dict) -> Path:
    import talib

    columns = read_columns(DAILY)
    outputs, lookback, functions = FORMULAS[name](columns, params)
    outputs = warm_up(outputs, lookback)
    if len(outputs) != len(spec["outputs"]):
        raise SystemExit(f"{name}: formula returned {len(outputs)} outputs, spec has {len(spec['outputs'])}")

    rendered = ", ".join(f"{key}={value}" for key, value in params.items()) or "none"
    header = [
        f"# indicator: {name}",
        f"# case: {case}",
        f"# params: {rendered}",
        f"# oracle: oracle A, the formula of INDICATORS.md section 5.3 evaluated through "
        f"ta-lib-python {talib.__version__} {functions}",
        f"# produced_by: python scripts/oracle/formula_golden.py {name} --case {case}",
        f"# input: testdata/{DAILY} (all rows)",
        f"# tolerance: {TOLERANCE}",
        "# excluded_rows: none",
        f"# note: rows before {lookback} are warm-up for every output (CONVENTIONS.md section 2)",
        f"# date: {dt.date.today().isoformat()}",
    ]
    names = [i["name"] for i in spec["inputs"]] + [o["name"] for o in spec["outputs"]]
    lines = [*header, ",".join(names)]
    for row in range(len(columns["close"])):
        cells = [repr(float(columns[i["name"]][row])) for i in spec["inputs"]]
        cells += [repr(float(values[row])) for values in outputs]
        lines.append(",".join(cells))

    path = INDICATORS / name / "golden" / f"{case}.csv"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    size = path.stat().st_size
    if size > 2_000_000:
        raise SystemExit(f"{path.name} is {size} bytes; docs/SPEC_FORMAT.md section 4 caps it at 2 MB")
    return path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("indicator", help="folder name under crates/trendlib/src/indicators")
    parser.add_argument("--case", help="one case; default is every case")
    args = parser.parse_args()

    try:
        import talib  # noqa: F401
        import yaml  # noqa: F401
    except ImportError as exc:
        raise SystemExit(f"{exc.name} is missing; install the dev extra: pip install -e '.[dev]'") from exc

    if args.indicator not in FORMULAS:
        raise SystemExit(f"{args.indicator} has no formula here; see docs/INDICATORS.md section 5.3")
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
