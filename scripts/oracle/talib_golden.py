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
}

# Which committed dataset a case reads. Session-anchored indicators need the
# intraday file; everything else runs on daily bars.
DAILY = "daily_2000.csv"
INTRADAY = "intraday_5m_20d.csv"


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
}


def tolerance_for(name: str, case: str) -> tuple[str, str | None]:
    """The tolerance a golden file carries, and the reason if it is not the default."""
    if (name, case) in CASE_TOLERANCE:
        return CASE_TOLERANCE[(name, case)]
    if name in INDICATOR_TOLERANCE:
        return INDICATOR_TOLERANCE[name]
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
    """The enum's values that have an indicator of the same name.

    The averages the library can actually build; the rest are approved but not
    implemented, and asking for one is an error rather than a golden case.
    """
    import yaml

    values = yaml.safe_load(ENUMS.read_text(encoding="utf-8"))[enum_name]["values"]
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
    every period-like parameter to its documented minimum, which is the
    boundary most likely to be off by one in an implementation.
    """
    found = {"default": defaults(spec)}
    floors = {
        param["name"]: param["min"]
        for param in spec.get("params") or []
        if param["type"] == "int" and param["name"].endswith("period") and "min" in param
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
    for (name, case), (period, _) in CASE_OVERRIDES.items():
        if name == spec["name"] and case in found:
            found[case] = {**found[case], "period": period}
    return found


def dataset_for(spec: dict) -> str:
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


# Functions whose domain the raw close prices fall outside of. An arc cosine of
# 1000 is NaN on every row and an exponential of 1000 is infinity on every row;
# either way the file would prove nothing, so the source is scaled into [-1, 1]
# first. The header records that it was.
SCALED_SOURCE = {"acos", "asin", "exp", "cosh", "sinh"}

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


def run_oracle(spec: dict, bound: dict, params: dict):
    """Call TA-Lib with its own parameter names and return its output arrays."""
    import talib

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

    dataset = dataset_for(spec)
    columns = read_dataset(dataset)
    bound = bind_inputs(spec, columns)
    outputs = run_oracle(spec, bound, params)

    expected = len(spec["outputs"])
    if len(outputs) != expected:
        raise SystemExit(
            f"{name}/{case}: oracle returned {len(outputs)} outputs, spec has {expected}"
        )

    rendered = ", ".join(f"{key}={value}" for key, value in params.items()) or "none"
    header = [
        f"# indicator: {name}",
        f"# case: {case}",
        f"# params: {rendered}",
        f"# oracle: ta-lib-python {talib.__version__} "
        f"(TA-Lib C {talib.__ta_version__.decode().split()[0]}), talib.{spec['talib']['name']}",
        f"# produced_by: python scripts/oracle/talib_golden.py {name} --case {case}",
        f"# input: testdata/{dataset} (all rows)"
        + (", source scaled into [-1, 1]" if name in SCALED_SOURCE else ""),
        f"# tolerance: {tolerance_for(name, case)[0]}",
        "# excluded_rows: none",
        *([f"# note: {CASE_OVERRIDES[(name, case)][1]}"] if (name, case) in CASE_OVERRIDES else []),
        *([f"# note: {reason}"] if (reason := tolerance_for(name, case)[1]) else []),
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
