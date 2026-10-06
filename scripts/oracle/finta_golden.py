"""Produce golden CSVs for the oracle-F indicators of INDICATORS.md section 5.4.

    python scripts/oracle/finta_golden.py wavetrend                 # every case
    python scripts/oracle/finta_golden.py wavetrend --case default  # one case

These indicators exist in no MIT-licensed library, so their oracle is finta
1.3, a test-only dependency (``docs/DECISIONS.md`` D19, ``docs/TESTING.md``
section 2). This script calls the ``finta.TA`` method the section 5.4 table
names and writes what it returns. Rows before the indicator's warm-up are
blanked, which is TrendLib's alignment rule rather than part of any value.
"""

from __future__ import annotations

import argparse
import datetime as dt
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
INDICATORS = REPO_ROOT / "crates" / "trendlib" / "src" / "indicators"
TESTDATA = REPO_ROOT / "testdata"

TOLERANCE = "rel=1e-10 abs=1e-12"
DAILY = "daily_2000.csv"

# TrendLib's names for finta's parameters, where they differ.
RENAMED = {"channel_length": "channel_lenght", "average_length": "average_lenght"}

# Each indicator: the finta method, the columns of its result in spec order (a
# Series has one), and the warm-up rows TrendLib leaves blank.
CALLS = {
    "wavetrend": ("WTO", ["WT1.", "WT2."], lambda p: 4),
    "ift_rsi": ("IFT_RSI", None, lambda p: p["wma_period"]),
    "vzo": ("VZO", None, lambda p: 0),
    "pivots_fibonacci": (
        "PIVOT_FIB",
        ["pivot", "s1", "s2", "s3", "s4", "r1", "r2", "r3", "r4"],
        lambda p: 1,
    ),
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
    found = {"default": defaults(spec)}
    floors = {
        param["name"]: param["min"]
        for param in spec.get("params") or []
        if param["type"] == "int" and "min" in param
    }
    if floors:
        found["min_period"] = {**defaults(spec), **floors}
    return found


def run_oracle(name: str, frame, params: dict):
    import numpy as np
    from finta import TA

    method, columns, _ = CALLS[name]
    keywords = {RENAMED.get(key, key): value for key, value in params.items()}
    result = getattr(TA, method)(frame.copy(), **keywords)
    if columns is None:
        return [np.array(result, dtype=np.float64)]
    return [np.array(result[column], dtype=np.float64) for column in columns]


def write_case(name: str, case: str, spec: dict, params: dict) -> Path:
    from importlib.metadata import version

    import numpy as np
    import pandas as pd

    frame = pd.read_csv(TESTDATA / DAILY)
    outputs = run_oracle(name, frame, params)
    lookback = CALLS[name][2](params)
    for values in outputs:
        values[:lookback] = np.nan
    if len(outputs) != len(spec["outputs"]):
        raise SystemExit(
            f"{name}: finta gave {len(outputs)} outputs, spec has {len(spec['outputs'])}"
        )

    method = CALLS[name][0]
    rendered = ", ".join(f"{key}={value}" for key, value in params.items()) or "none"
    header = [
        f"# indicator: {name}",
        f"# case: {case}",
        f"# params: {rendered}",
        f"# oracle: finta {version('finta')}, finta.TA.{method} (oracle F)",
        f"# produced_by: python scripts/oracle/finta_golden.py {name} --case {case}",
        f"# input: testdata/{DAILY} (all rows)",
        f"# tolerance: {TOLERANCE}",
        "# excluded_rows: none",
        f"# note: rows before {lookback} are warm-up for every output (CONVENTIONS.md section 2)",
        f"# date: {dt.date.today().isoformat()}",
    ]
    columns = {"source": "close"}
    names = [i["name"] for i in spec["inputs"]] + [o["name"] for o in spec["outputs"]]
    inputs = [
        frame[columns.get(i["name"], i["name"])].to_numpy(dtype=np.float64) for i in spec["inputs"]
    ]
    lines = [*header, ",".join(names)]
    for row in range(len(frame)):
        cells = [repr(float(column[row])) for column in inputs]
        cells += [repr(float(values[row])) for values in outputs]
        lines.append(",".join(cells))

    path = INDICATORS / name / "golden" / f"{case}.csv"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    return path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("indicator", help="folder name under crates/trendlib/src/indicators")
    parser.add_argument("--case", help="one case; default is every case")
    args = parser.parse_args()

    try:
        import finta  # noqa: F401
        import yaml  # noqa: F401
    except ImportError as exc:
        raise SystemExit(
            f"{exc.name} is missing; install the dev extra: pip install -e '.[dev]'"
        ) from exc

    if args.indicator not in CALLS:
        raise SystemExit(f"{args.indicator} is not on oracle F; see docs/INDICATORS.md section 5.4")
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
