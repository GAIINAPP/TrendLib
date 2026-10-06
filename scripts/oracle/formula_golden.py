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

# Cases run at the defaults on another dataset, for a branch of the formula the
# daily walk never reaches. patterns_1080.csv has bars that close at their open.
OTHER_DATA = {
    "pivots_demark": {"patterns": "patterns_1080.csv"},
}


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
    for case in OTHER_DATA.get(spec["name"], {}):
        found[case] = defaults(spec)
    return found


def dataset_for(name: str, case: str) -> str:
    return OTHER_DATA.get(name, {}).get(case, DAILY)


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

    return (
        talib.MIDPRICE(high, low, timeperiod=period) if period >= 2 else talib.MEDPRICE(high, low)
    )


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


def constant(like, value: float):
    import numpy as np

    return np.full_like(like, value)


def smma(values, period: int):
    """Williams' smoothed average: TA-Lib's SMA of the first `period`, then each bar
    moves it `1 / period` of the way to the new value."""
    import numpy as np
    import talib

    out = talib.SMA(values, timeperiod=period)
    if period == 1:
        return out
    k = 1.0 / period
    start = int(np.flatnonzero(~np.isnan(out))[0])
    for t in range(start + 1, len(values)):
        out[t] = (values[t] - out[t - 1]) * k + out[t - 1]
    return out


def alligator_lines(cols, p):
    import talib

    median = talib.MEDPRICE(cols["high"], cols["low"])
    names = ("jaw", "teeth", "lips")
    lines = [shifted(smma(median, p[f"{n}_period"]), p[f"{n}_shift"]) for n in names]
    return lines, max(p[f"{n}_period"] - 1 + p[f"{n}_shift"] for n in names)


def alligator(cols, p):
    lines, lookback = alligator_lines(cols, p)
    return lines, lookback, "talib.MEDPRICE and SMA (the seed), the smoothing step in NumPy"


def gator(cols, p):
    import numpy as np
    import talib

    (jaw, teeth, lips), lookback = alligator_lines(cols, p)
    outputs = [np.abs(talib.SUB(jaw, teeth)), -np.abs(talib.SUB(teeth, lips))]
    return outputs, lookback, "talib.MEDPRICE, SMA (the seed) and SUB, the smoothing step in NumPy"


def envelope(cols, p):
    import talib

    middle = talib.SMA(cols["source"], timeperiod=p["period"])
    upper = talib.MULT(middle, constant(middle, 1.0 + p["percent"] / 100.0))
    lower = talib.MULT(middle, constant(middle, 1.0 - p["percent"] / 100.0))
    return [upper, middle, lower], p["period"] - 1, "talib.SMA and MULT"


GUPPY = (3, 5, 8, 10, 12, 15, 30, 35, 40, 45, 50, 60)


def guppy(cols, p):
    import talib

    lines = [talib.EMA(cols["source"], timeperiod=n) for n in GUPPY]
    return lines, max(GUPPY) - 1, "talib.EMA"


def woodies_cci(cols, p):
    import talib

    hlc = (cols["high"], cols["low"], cols["close"])
    outputs = [
        talib.CCI(*hlc, timeperiod=p["cci_period"]),
        talib.CCI(*hlc, timeperiod=p["turbo_period"]),
    ]
    return outputs, max(p["cci_period"], p["turbo_period"]) - 1, "talib.CCI"


def atr_bands(cols, p):
    import talib

    average = talib.ATR(cols["high"], cols["low"], cols["close"], timeperiod=p["period"])
    width = talib.MULT(average, constant(average, p["shift"]))
    outputs = [talib.ADD(cols["close"], width), talib.SUB(cols["close"], width)]
    return outputs, p["period"], "talib.ATR, MULT, ADD and SUB"


def pivots_woodie(cols, p):
    import talib

    high, low, close = (shifted(cols[k], 1) for k in ("high", "low", "close"))
    two = constant(close, 2.0)
    pivot = talib.DIV(talib.ADD(talib.ADD(high, low), talib.MULT(two, close)), constant(close, 4.0))
    span = talib.SUB(high, low)
    outputs = [
        pivot,
        talib.SUB(talib.MULT(two, pivot), low),
        talib.SUB(talib.MULT(two, pivot), high),
        talib.ADD(pivot, span),
        talib.SUB(pivot, span),
    ]
    return outputs, 1, "talib.ADD, SUB, MULT and DIV"


def pivots_demark(cols, p):
    import numpy as np

    opened, high, low, close = (shifted(cols[k], 1) for k in ("open", "high", "low", "close"))
    total = np.where(
        close < opened,
        high + 2.0 * low + close,
        np.where(close > opened, 2.0 * high + low + close, high + low + 2.0 * close),
    )
    outputs = [total / 4.0, total / 2.0 - low, total / 2.0 - high]
    return outputs, 1, "nothing: NumPy arithmetic, as no TA-Lib function computes a step"


FORMULAS = {
    "ichimoku": ichimoku,
    "alligator": alligator,
    "gator": gator,
    "envelope": envelope,
    "guppy": guppy,
    "woodies_cci": woodies_cci,
    "atr_bands": atr_bands,
    "pivots_woodie": pivots_woodie,
    "pivots_demark": pivots_demark,
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

    dataset = dataset_for(name, case)
    columns = read_columns(dataset)
    outputs, lookback, functions = FORMULAS[name](columns, params)
    outputs = warm_up(outputs, lookback)
    if len(outputs) != len(spec["outputs"]):
        raise SystemExit(
            f"{name}: formula returned {len(outputs)} outputs, spec has {len(spec['outputs'])}"
        )

    rendered = ", ".join(f"{key}={value}" for key, value in params.items()) or "none"
    header = [
        f"# indicator: {name}",
        f"# case: {case}",
        f"# params: {rendered}",
        f"# oracle: oracle A, the formula of INDICATORS.md section 5.3 evaluated through "
        f"ta-lib-python {talib.__version__} {functions}",
        f"# produced_by: python scripts/oracle/formula_golden.py {name} --case {case}",
        f"# input: testdata/{dataset} (all rows)",
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
        import talib  # noqa: F401
        import yaml  # noqa: F401
    except ImportError as exc:
        raise SystemExit(
            f"{exc.name} is missing; install the dev extra: pip install -e '.[dev]'"
        ) from exc

    if args.indicator not in FORMULAS:
        raise SystemExit(
            f"{args.indicator} has no formula here; see docs/INDICATORS.md section 5.3"
        )
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
