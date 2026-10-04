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


@pytest.fixture(scope="session")
def closes(testdata):
    """Close prices from the committed daily dataset."""
    import csv

    with (testdata / "daily_2000.csv").open(newline="", encoding="utf-8") as handle:
        return np.array([float(row["close"]) for row in csv.DictReader(handle)])


@pytest.fixture(params=["sma", "ema", "rsi"])
def indicator(request):
    return request.param
