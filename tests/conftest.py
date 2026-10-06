"""Shared fixtures.

The `indicator` fixture enumerates every indicator the extension module
exposes, so a contract test written once covers the whole catalogue and a newly
added indicator is tested the moment its spec exists. Nothing here lists
indicator names by hand.
"""

import csv
from pathlib import Path

import numpy as np
import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
TESTDATA = REPO_ROOT / "testdata"


@pytest.fixture(scope="session")
def repo_root() -> Path:
    return REPO_ROOT


@pytest.fixture(scope="session")
def testdata() -> Path:
    return TESTDATA


def bitwise_equal(left, right) -> bool:
    """True when two float64 arrays are identical bar for bar, NaN included.

    Value equality would accept a -0.0 where the other has +0.0; the parity
    guarantee in docs/PYTHON_API.md section 4 is bitwise, so this compares bits
    and treats any NaN as equal to any NaN.
    """
    left = np.asarray(left, dtype=np.float64)
    right = np.asarray(right, dtype=np.float64)
    if left.shape != right.shape:
        return False
    both_nan = np.isnan(left) & np.isnan(right)
    same_bits = left.view(np.uint64) == right.view(np.uint64)
    return bool(np.all(both_nan | same_bits))


def warm_up(column) -> np.ndarray:
    """Rows an indicator has not produced a value for yet.

    Float outputs warm up with NaN and integer ones with zero
    (`docs/CONVENTIONS.md` section 2), so `np.isnan` alone answers False for
    every row of an integer output and would quietly assert nothing.
    """
    column = np.asarray(column)
    if column.dtype.kind in "iu":
        return column == 0
    return np.isnan(column)


def _daily() -> dict[str, np.ndarray]:
    with (TESTDATA / "daily_2000.csv").open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    columns = {
        name: np.array([float(row[name]) for row in rows])
        for name in ("open", "high", "low", "close", "volume")
    }
    # The same mapping the oracle script and the Rust suites use, so every
    # layer is looking at the same bars.
    # The daily dataset carries no timestamps, so one bar a day is synthesised.
    # `vwap`'s session behaviour is pinned by its golden files, which use the
    # intraday dataset and its real timestamps.
    columns["timestamps"] = np.array(
        [1_767_225_600_000_000_000 + row * 86_400_000_000_000 for row in range(len(rows))],
        dtype=np.float64,
    )
    # `mavp` asks for a period per bar rather than a price; this is the same
    # ramp the oracle script writes into its golden files.
    columns["periods"] = np.array([2.0 + (row % 29) for row in range(len(rows))])
    columns["source"] = columns["close"]
    columns["source0"] = columns["close"]
    columns["source1"] = columns["open"]
    return columns


@pytest.fixture(scope="session")
def bars() -> dict[str, np.ndarray]:
    return _daily()


@pytest.fixture(scope="session")
def closes(bars) -> np.ndarray:
    return bars["close"]


class Indicator:
    """One indicator, with everything a test needs to call it."""

    def __init__(self, name: str) -> None:
        from trendlib import _core

        self.name = name
        self.inputs: list[str] = list(_core.INPUTS[name])
        self.kinds: list[str] = list(_core.KINDS[name])
        self.outputs: list[str] = list(_core.OUTPUTS[name])
        self.params: dict = dict(_core.PARAMS[name])
        self.group: str = _core.GROUPS[name]
        self.flags: list[str] = list(_core.FLAGS[name])
        self.dtypes: list[str] = list(_core.DTYPES[name])

    @property
    def absolute_index(self) -> bool:
        """Outputs are row indices, so a warm-up prefix shifts them."""
        return "absolute_index" in self.flags

    @property
    def multi_output(self) -> bool:
        return len(self.outputs) > 1

    def columns(self, bars, rows: int | None = None) -> list[np.ndarray]:
        return [bars[name][:rows] for name in self.inputs]

    def frame(self, bars, rows: int | None = None):
        """A DataFrame holding every column this indicator could want."""
        import pandas as pd

        held = {name: bars[name][:rows] for name in ("open", "high", "low", "close", "volume")}
        return pd.DataFrame(held)

    def defaults(self) -> dict:
        return {name: spec["default"] for name, spec in self.params.items()}

    def call(self, bars, rows: int | None = None, **params):
        import trendlib as tl

        return getattr(tl, self.name)(*self.columns(bars, rows), **params)

    def lookback(self, **params) -> int:
        import trendlib as tl

        return tl.lookback(self.name, **params)

    def __repr__(self) -> str:
        return self.name


def _names() -> list[str]:
    from trendlib import _core

    return sorted(_core.INPUTS)


@pytest.fixture(params=_names())
def indicator(request) -> Indicator:
    return Indicator(request.param)


@pytest.fixture(scope="session")
def every_indicator() -> list[Indicator]:
    return [Indicator(name) for name in _names()]


def pytest_addoption(parser):
    parser.addoption(
        "--bench-bars",
        type=int,
        default=1_000_000,
        help="bars the benchmarks in tests/bench run on (docs/TESTING.md section 8)",
    )
