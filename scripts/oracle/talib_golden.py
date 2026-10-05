"""Produce golden CSVs by running ta-lib-python, the T oracle.

    python scripts/oracle/talib_golden.py rsi                 # every case
    python scripts/oracle/talib_golden.py rsi --case default  # one case

The expected values in a golden file must come from an implementation that is
independent of TrendLib's (``docs/TESTING.md`` section 2). This script calls
TA-Lib and writes what TA-Lib returns, unchanged. It never computes an expected
value itself, and `cargo xtask golden` runs it rather than anything in the Rust
tree.

The parameters of each case, the dataset, the oracle version and the tolerance
all go into the file header, so a reader can reproduce it without reading this
script (``docs/SPEC_FORMAT.md`` section 4).
"""

from __future__ import annotations

import argparse
import csv
import datetime as dt
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
INDICATORS = REPO_ROOT / "crates" / "trendlib" / "src" / "indicators"
ENUMS = INDICATORS / "_enums.yaml"
TESTDATA = REPO_ROOT / "testdata"

TOLERANCE = "rel=1e-10 abs=1e-12"

# Cases where the oracle, not TrendLib, is the less accurate of the two, so the
# default tolerance would be measuring TA-Lib's error rather than ours.
# `docs/SPEC_FORMAT.md` section 4 allows a looser per-file tolerance with a
# written reason; the reason goes in the header beside it.
SLOPE_CANCELS = (
    "the fitted slope is the difference of two sums of the same size, so almost every digit "
    "cancels; measured against exact arithmetic over this dataset at the default period "
    "TrendLib's worst relative error is 2.8e-11 and ta-lib-python's is 8.9e-10, and TrendLib "
    "is the closer of the two on 1896 rows to 45, so the looser bound covers the oracle's own "
    "error rather than TrendLib's"
)

SLOPE_CANCELS_AT_TWO = (
    "over two bars the fitted slope is the difference between them and nothing else survives "
    "the cancellation; measured against exact arithmetic over this dataset TrendLib's worst "
    "relative error is 2.8e-10 and ta-lib-python's is 8.0e-8, with TrendLib the closer of the "
    "two on 1911 rows to 9, so the looser bound covers the oracle's own error rather than "
    "TrendLib's"
)

PAIRED_AT_TWO = (
    "over two bars a correlation is exactly +1 or -1 and a beta is exactly the ratio of the two "
    "returns, and almost every digit cancels on the way there; measured against those exact "
    "values over this dataset TrendLib reaches the correlation exactly on 1384 rows where "
    "ta-lib-python reaches it on 981 and strays 7.2e-10, and TrendLib's worst relative error on "
    "beta is 4.4e-14 against ta-lib-python's 1.2e-9, closer on 1852 rows to 45, so the looser "
    "bound covers the oracle's own error rather than TrendLib's"
)

FORECAST_CANCELS = (
    "the value is the gap between a bar and the line fitted to the bars before it, so a price-"
    "sized error of 1e-13 arrives as 1e-9 on a gap of 0.004; measured against exact arithmetic "
    "over this dataset TrendLib's worst relative error is 3.1e-11 and ta-lib-python's is 2.2e-9, "
    "with TrendLib the closer of the two on 1896 rows to 64, so the looser bound covers the "
    "oracle's own error rather than TrendLib's"
)

TRIMA_RESIDUE = (
    "%K swings between 0 and 100 and back, and a triangular average carries that swing in two "
    "running sums whose residue never cancels; measured against exact arithmetic over this "
    "dataset TrendLib's worst absolute error is 3.1e-13 and ta-lib-python's is 5.8e-11, with "
    "TrendLib the closer of the two on 1986 rows to 6, so the looser bound covers the oracle's "
    "own error rather than TrendLib's"
)

CASE_TOLERANCE = {
    ("stddev", "min_period"): (
        "rel=1e-9 abs=1e-12",
        "at period 2 the population standard deviation is |a - b| / 2; measured against "
        "that exact value over the dataset TrendLib is exact on 98.7 percent of rows and "
        "never worse than 1.6e-16, while ta-lib-python reaches 3.2e-10, so the looser "
        "bound covers the oracle's own error rather than TrendLib's",
    ),
    ("stochf", "fastd_ma_type_trima"): ("rel=1e-8 abs=2e-10", TRIMA_RESIDUE),
    ("stoch", "slowk_ma_type_trima"): ("rel=1e-8 abs=2e-10", TRIMA_RESIDUE),
    ("stoch", "slowd_ma_type_trima"): ("rel=1e-8 abs=2e-10", TRIMA_RESIDUE),
    ("stochrsi", "fastd_ma_type_trima"): ("rel=1e-8 abs=2e-10", TRIMA_RESIDUE),
    ("kdj", "slowk_ma_type_trima"): ("rel=1e-8 abs=5e-10", TRIMA_RESIDUE),
    ("kdj", "slowd_ma_type_trima"): ("rel=1e-8 abs=5e-10", TRIMA_RESIDUE),
    ("correl", "min_period"): ("rel=1e-8 abs=1e-12", PAIRED_AT_TWO),
    ("beta", "min_period"): ("rel=1e-5 abs=1e-12", PAIRED_AT_TWO),
    ("linearreg_slope", "min_period"): ("rel=2e-7 abs=1e-12", SLOPE_CANCELS_AT_TWO),
    ("linearreg_angle", "min_period"): ("rel=2e-7 abs=1e-12", SLOPE_CANCELS_AT_TWO),
}

# Which committed dataset a case reads. Session-anchored indicators need the
# intraday file; everything else runs on daily bars.
DAILY = "daily_2000.csv"
INTRADAY = "intraday_5m_20d.csv"
PATTERNS = "patterns_1080.csv"


# Indicators where cancellation, not an error on either side, puts the two
# implementations past the default bound. `docs/SPEC_FORMAT.md` section 4 allows
# a looser per-file tolerance with a written reason; the reason goes in the
# header beside the numbers, and the Rust edge-case suite pins the properties
# that do not cancel so the looser bound cannot hide a regression.
CANCELLATION = (
    "the output is a difference of two moving averages of the same series and is around 1e-5 "
    "of them, so one ULP on either average lands as ~1e-11 on the difference; measured against "
    "exact arithmetic over this dataset TrendLib's worst relative error is 4.7e-10 (rma) and "
    "ta-lib-python's is 1.1e-9 (trima), with TrendLib the closer of the two on wma and trima "
    "by a factor of ten, so this bound covers the two errors added rather than either "
    "implementation being wrong"
)

INDICATOR_TOLERANCE = {
    "apo": ("rel=2e-9 abs=1e-12", CANCELLATION),
    "ppo": ("rel=2e-9 abs=1e-12", CANCELLATION),
    "linearreg_slope": ("rel=2e-9 abs=1e-12", SLOPE_CANCELS),
    "linearreg_angle": ("rel=2e-9 abs=1e-12", SLOPE_CANCELS),
    "fosc": ("rel=5e-9 abs=1e-12", FORECAST_CANCELS),
    "ht_phasor": (
        "rel=1e-10 abs=1e-10",
        "the two parts cross zero, and a relative bound measures nothing within a hair of a "
        "crossing; the worst absolute disagreement over this dataset is 1.4e-11 against "
        "quantities that run to 134, so the absolute bound is the one that says anything",
    ),
}


# Tolerances for an indicator's enum cases only. Where the default case
# reproduces the oracle exactly there is no reason to loosen it as well, and
# leaving it tight is what would catch a regression in the shared machinery.
ENUM_CASE_TOLERANCE = {
    "macdext": (
        "rel=1e-8 abs=1e-11",
        "the line is a difference of two averages of the series and the signal and histogram "
        "are differences of that again, so an error of 1e-13 on a value near 1000 arrives as "
        "4e-9 on a histogram near 0.05; with the default averages TrendLib reproduces "
        "ta-lib-python to the last bit, which is what the default and min_period cases hold it "
        "to",
    ),
}

BASE_CASES = ("default", "min_period")


def tolerance_for(name: str, case: str) -> tuple[str, str | None]:
    """The tolerance a golden file carries, and the reason if it is not the default."""
    if (name, case) in CASE_TOLERANCE:
        return CASE_TOLERANCE[(name, case)]
    if name in INDICATOR_TOLERANCE:
        return INDICATOR_TOLERANCE[name]
    if name in ENUM_CASE_TOLERANCE and case not in BASE_CASES:
        return ENUM_CASE_TOLERANCE[name]
    return TOLERANCE, None


def load_spec(name: str) -> dict:
    import yaml

    path = INDICATORS / name / "spec.yaml"
    if not path.exists():
        raise SystemExit(f"no spec at {path.relative_to(REPO_ROOT)}")
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def defaults(spec: dict) -> dict:
    return {param["name"]: param["default"] for param in spec.get("params") or []}


# A boundary case whose documented minimum the oracle cannot be trusted at.
# The value is the period to use instead and the reason, which goes in the file
# header so the substitution is visible where the numbers are.
CASE_OVERRIDES = {
    ("plus_di", "min_period"): (
        2,
        "TA-Lib returns the raw fraction at period=1 rather than scaling it to a "
        "percentage (CONVENTIONS.md deviation 7), so the oracle is only usable from "
        "period=2",
    ),
    ("minus_di", "min_period"): (
        2,
        "TA-Lib returns the raw fraction at period=1 rather than scaling it to a "
        "percentage (CONVENTIONS.md deviation 7), so the oracle is only usable from "
        "period=2",
    ),
    ("natr", "min_period"): (
        2,
        "TA-Lib returns the raw true range at period=1 rather than normalising it "
        "(CONVENTIONS.md deviation 6), so the oracle is only usable from period=2",
    ),
}


def shipped_values(enum_name: str) -> list[str]:
    """The values of `enum_name` an indicator can actually be asked for.

    `MaType` offers the averages the library has built, which are the ones with
    an indicator of the same name; the rest are approved but not implemented,
    and asking for one is an error rather than a golden case. Every other
    enum's values stand on their own.
    """
    import yaml

    values = yaml.safe_load(ENUMS.read_text(encoding="utf-8"))[enum_name]["values"]
    if enum_name != "MaType":
        return values
    return [value for value in values if (INDICATORS / value / "spec.yaml").exists()]


def talib_value(spec: dict, name: str, value):
    """A parameter value as ta-lib-python wants it.

    An enum is a name here and one of TA-Lib's integers there; `_enums.yaml`
    owns the mapping so this script does not carry a second copy of it.
    """
    import yaml

    declared = {param["name"]: param for param in spec.get("params") or []}
    kind = declared.get(name, {}).get("type", "")
    if not kind.startswith("enum:"):
        return value
    numbers = yaml.safe_load(ENUMS.read_text(encoding="utf-8"))[kind.split(":", 1)[1]]
    return numbers["talib_int"][value]


def cases(spec: dict) -> dict[str, dict]:
    """Every parameter case a golden file is written for.

    `default` is required by docs/SPEC_FORMAT.md section 4. `min_period` pins
    every whole-numbered parameter to its documented minimum, which is the
    boundary most likely to be off by one in an implementation. It keeps that
    name whatever the parameters are called: `fractal` counts bars either side
    of a swing rather than a period, and the case means the same thing.
    """
    found = {"default": defaults(spec)}
    floors = {
        param["name"]: param["min"]
        for param in spec.get("params") or []
        if param["type"] == "int" and "min" in param
    }
    if floors:
        found["min_period"] = {**defaults(spec), **floors}
    for param in spec.get("params") or []:
        if not param["type"].startswith("enum:"):
            continue
        for value in shipped_values(param["type"].split(":", 1)[1]):
            if value == param["default"]:
                continue
            found[f"{param['name']}_{value}"] = {**defaults(spec), param["name"]: value}
    # A pattern fires on a handful of bars or on none at all, and on the daily
    # dataset thirty-one of the sixty-one fire fewer than five times. The
    # second case reads bars built to make each one fire, so the file says what
    # the pattern recognises rather than only when it stays quiet.
    if "pattern" in (spec.get("flags") or []):
        found["patterns"] = defaults(spec)
    for (name, case), (period, _) in CASE_OVERRIDES.items():
        if name == spec["name"] and case in found:
            found[case] = {**found[case], "period": period}
    return found


def dataset_for(spec: dict, case: str = "default") -> str:
    if case == "patterns":
        return PATTERNS
    kinds = {i["kind"] for i in spec["inputs"]}
    return INTRADAY if "timestamps" in kinds else DAILY


def read_dataset(filename: str) -> dict[str, list]:
    path = TESTDATA / filename
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    columns: dict[str, list] = {key: [] for key in rows[0]}
    for row in rows:
        for key, value in row.items():
            columns[key].append(value)
    return columns


# Functions the raw close prices tell nothing about. An arc cosine of 1000 is
# NaN on every row, an exponential of 1000 is infinity on every row, and a
# hyperbolic tangent of 1000 is 1.0 on every row: a file of one repeated value
# proves nothing, so the source is scaled into [-1, 1] first. The header records
# that it was.
SCALED_SOURCE = {"acos", "asin", "exp", "cosh", "sinh", "tanh"}

# The two operands of an arithmetic operator want to be different series, or
# `div` is 1.0 on every row and `sub` is 0.0 on every row.
SECOND_OPERAND = "open"


def scale_to_unit(values):
    import numpy as np

    low, high = float(np.min(values)), float(np.max(values))
    if high == low:
        return np.zeros_like(values)
    return (values - low) / (high - low) * 2.0 - 1.0


def bind_inputs(spec: dict, columns: dict[str, list]):
    """Map the spec's input names onto dataset columns."""
    import numpy as np

    bound = {}
    for spec_input in spec["inputs"]:
        name, kind = spec_input["name"], spec_input["kind"]
        if name == "periods":
            # A column of prices would clamp to the longest period on every
            # bar, which tests the clamp and nothing else. This walks the
            # whole range instead, deterministically, and the values are
            # written into the golden file like any other input.
            bound[name] = np.array(
                [2.0 + (row % 29) for row in range(len(columns["close"]))],
                dtype=np.float64,
            )
            continue
        if kind == "series":
            source = SECOND_OPERAND if name == "source1" else "close"
        elif kind == "timestamps":
            source = "timestamp"
        else:
            source = kind
        if source not in columns:
            raise SystemExit(f"dataset has no column {source!r} for input {name!r}")
        if kind == "timestamps":
            bound[name] = np.array([int(v) for v in columns[source]], dtype=np.int64)
        else:
            values = np.array([float(v) for v in columns[source]], dtype=np.float64)
            if spec["name"] in SCALED_SOURCE:
                values = scale_to_unit(values)
            bound[name] = values
    return bound


def session_vwap(bound: dict, anchor: str):
    """TA-Lib's VWAP run separately on each session's slice.

    TA-Lib's own VWAP never resets, so the only way to get the session-anchored
    figure out of it is to hand it one session at a time. The slices are cut on
    the calendar date of the timestamp, which is what `anchor="day"` means
    (`INDICATORS.md` section 3.1).
    """
    import numpy as np
    import talib

    high, low, close = bound["high"], bound["low"], bound["close"]
    volume, stamps = bound["volume"], bound["timestamps"]
    out = np.full(len(close), np.nan)
    if anchor == "none":
        return talib.VWAP(high, low, close, volume)
    day = stamps // 86_400_000_000_000
    for session in np.unique(day):
        rows = np.flatnonzero(day == session)
        out[rows] = talib.VWAP(high[rows], low[rows], close[rows], volume[rows])
    return out


def excluded_rows(spec: dict, bound: dict, params: dict) -> tuple[str, str | None]:
    """Rows the oracle cannot pin, as a range list and the reason.

    Only `vwap` has any: before volume has traded in a session TA-Lib carries
    the previous value or starts at zero, and zero is not a price
    (`CONVENTIONS.md` deviation 2).
    """
    import numpy as np

    if spec["name"] != "vwap":
        return "none", None
    volume, stamps = bound["volume"], bound["timestamps"]
    day = stamps // 86_400_000_000_000 if params["anchor"] == "day" else np.zeros_like(stamps)
    traded = np.zeros(len(volume))
    for session in np.unique(day):
        rows = np.flatnonzero(day == session)
        traded[rows] = np.cumsum(volume[rows])
    rows = np.flatnonzero(traded == 0.0)
    if len(rows) == 0:
        return "none", None
    ranges = []
    start = previous = int(rows[0])
    for row in map(int, rows[1:]):
        if row != previous + 1:
            ranges.append((start, previous))
            start = row
        previous = row
    ranges.append((start, previous))
    listed = ", ".join(f"{low}-{high}" for low, high in ranges)
    return (
        f"{listed} (Deviation 2)",
        "before any volume has traded in a session there is no average fill to report, and "
        "TA-Lib answers zero where this answers NaN, because zero is not a price",
    )


#: What the levels are: TA-Lib has no function for them, so the published
#: formulas are evaluated through TA-Lib's own arithmetic functions. Every
#: value comes out of the C library; only the shape of the expression is ours,
#: and it is the one `INDICATORS.md` section 3 approves. Oracle A, weaker than
#: a second end-to-end implementation and stronger than nothing; a transcribed
#: reference (`DECISIONS.md` Q4) would strengthen it further.
LEVELS = {"cpr", "pivots_traditional", "pivots_camarilla"}


def level_oracle(name: str, bound: dict):
    """The levels, evaluated with TA-Lib's arithmetic over the previous bar."""
    import numpy as np
    import talib

    # Row t describes the period that starts at t, built from bar t - 1.
    def earlier(values):
        shifted = np.full(len(values), np.nan)
        shifted[1:] = values[:-1]
        return shifted

    high, low, close = (earlier(bound[k]) for k in ("high", "low", "close"))
    two = np.full(len(close), 2.0)
    pivot = talib.TYPPRICE(high, low, close)
    if name == "cpr":
        bottom = talib.MEDPRICE(high, low)
        return [pivot, bottom, talib.SUB(talib.MULT(two, pivot), bottom)]
    span = talib.SUB(high, low)
    if name == "pivots_traditional":
        return [
            pivot,
            talib.SUB(talib.MULT(two, pivot), low),
            talib.ADD(pivot, span),
            talib.ADD(high, talib.MULT(two, talib.SUB(pivot, low))),
            talib.SUB(talib.MULT(two, pivot), high),
            talib.SUB(pivot, span),
            talib.SUB(low, talib.MULT(two, talib.SUB(high, pivot))),
        ]
    reach = talib.MULT(np.full(len(close), 1.1), span)
    steps = [talib.DIV(reach, np.full(len(close), divisor)) for divisor in (12.0, 6.0, 4.0, 2.0)]
    return [talib.ADD(close, step) for step in steps] + [talib.SUB(close, step) for step in steps]


def run_oracle(spec: dict, bound: dict, params: dict):
    """Call TA-Lib with its own parameter names and return its output arrays."""
    import talib

    if spec["name"] in LEVELS:
        return level_oracle(spec["name"], bound)
    if spec["name"] == "vwap":
        return [session_vwap(bound, params["anchor"])]
    alias = spec.get("talib")
    if alias is None:
        raise SystemExit(
            f"{spec['name']} has no talib: block, so ta-lib-python is not its oracle; "
            "see docs/TESTING.md section 2 for the human-transcribed route"
        )

    renames = alias.get("params") or {}
    talib_params = {
        renames.get(key, key): talib_value(spec, key, value) for key, value in params.items()
    }
    function = getattr(talib, alias["name"])
    result = function(*bound.values(), **talib_params)
    return list(result) if isinstance(result, tuple) else [result]


def write_case(name: str, case: str, spec: dict, params: dict) -> Path:
    import talib

    dataset = dataset_for(spec, case)
    columns = read_dataset(dataset)
    bound = bind_inputs(spec, columns)
    outputs = run_oracle(spec, bound, params)

    expected = len(spec["outputs"])
    if len(outputs) != expected:
        raise SystemExit(
            f"{name}/{case}: oracle returned {len(outputs)} outputs, spec has {expected}"
        )

    rendered = ", ".join(f"{key}={value}" for key, value in params.items()) or "none"
    excluded, excluded_why = excluded_rows(spec, bound, params)
    header = [
        f"# indicator: {name}",
        f"# case: {case}",
        f"# params: {rendered}",
        f"# oracle: ta-lib-python {talib.__version__} "
        f"(TA-Lib C {talib.__ta_version__.decode().split()[0]}), "
        + (
            "the formulas of INDICATORS.md section 3 evaluated through talib.TYPPRICE, "
            "MEDPRICE, ADD, SUB, MULT and DIV"
            if name in LEVELS
            else f"talib.{spec['talib']['name']}"
        ),
        f"# produced_by: python scripts/oracle/talib_golden.py {name} --case {case}",
        f"# input: testdata/{dataset} (all rows)"
        + (", source scaled into [-1, 1]" if name in SCALED_SOURCE else ""),
        f"# tolerance: {tolerance_for(name, case)[0]}",
        f"# excluded_rows: {excluded}",
        *([f"# note: {CASE_OVERRIDES[(name, case)][1]}"] if (name, case) in CASE_OVERRIDES else []),
        *([f"# note: {reason}"] if (reason := tolerance_for(name, case)[1]) else []),
        *([f"# note: {excluded_why}"] if excluded_why else []),
        f"# date: {dt.date.today().isoformat()}",
    ]

    names = [i["name"] for i in spec["inputs"]] + [o["name"] for o in spec["outputs"]]
    series = [bound[i["name"]] for i in spec["inputs"]] + outputs
    lines = [*header, ",".join(names)]
    lines.extend(
        ",".join(repr(column[row].item()) for column in series) for row in range(len(series[0]))
    )

    path = INDICATORS / name / "golden" / f"{case}.csv"
    path.parent.mkdir(parents=True, exist_ok=True)
    text = "\n".join(lines) + "\n"
    path.write_text(text, encoding="utf-8", newline="\n")

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

    spec = load_spec(args.indicator)
    available = cases(spec)
    wanted = {args.case: available[args.case]} if args.case else available
    if args.case and args.case not in available:
        raise SystemExit(f"unknown case {args.case!r}; known cases: {', '.join(available)}")

    for case, params in wanted.items():
        path = write_case(args.indicator, case, spec, params)
        print(f"wrote {path.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
