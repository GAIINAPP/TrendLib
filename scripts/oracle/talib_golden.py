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
TESTDATA = REPO_ROOT / "testdata"

TOLERANCE = "rel=1e-10 abs=1e-12"

# Which committed dataset a case reads. Session-anchored indicators need the
# intraday file; everything else runs on daily bars.
DAILY = "daily_2000.csv"
INTRADAY = "intraday_5m_20d.csv"


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
    ("natr", "min_period"): (
        2,
        "TA-Lib returns the raw true range at period=1 rather than normalising it "
        "(CONVENTIONS.md deviation 6), so the oracle is only usable from period=2",
    ),
}


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


def bind_inputs(spec: dict, columns: dict[str, list]):
    """Map the spec's input names onto dataset columns."""
    import numpy as np

    bound = {}
    for spec_input in spec["inputs"]:
        name, kind = spec_input["name"], spec_input["kind"]
        if kind == "series":
            source = "close"
        elif kind == "timestamps":
            source = "timestamp"
        else:
            source = kind
        if source not in columns:
            raise SystemExit(f"dataset has no column {source!r} for input {name!r}")
        if kind == "timestamps":
            bound[name] = np.array([int(v) for v in columns[source]], dtype=np.int64)
        else:
            bound[name] = np.array([float(v) for v in columns[source]], dtype=np.float64)
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
    talib_params = {renames.get(key, key): value for key, value in params.items()}
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
        f"# input: testdata/{dataset} (all rows)",
        f"# tolerance: {TOLERANCE}",
        "# excluded_rows: none",
        *([f"# note: {CASE_OVERRIDES[(name, case)][1]}"] if (name, case) in CASE_OVERRIDES else []),
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
