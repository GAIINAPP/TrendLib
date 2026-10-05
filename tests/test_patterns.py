"""Every candlestick pattern, on bars where it actually fires.

A pattern golden computed from the daily dataset is mostly zeros: over those
two thousand bars thirty-one of the sixty-one patterns fire fewer than five
times and twelve never fire at all, so the file proves only that the pattern
stayed silent. A rule can be wrong in a way such a file cannot see, and nine
were. The `patterns` case reads bars built to make each pattern fire
(`scripts/testdata/make_synthetic.py`); the tests here assert that it does,
and that agreement with the oracle holds on random bars too.
"""

import csv

import numpy as np
import pytest

import trendlib as tl
from trendlib import _core

talib = pytest.importorskip("talib", reason="the TA-Lib oracle is not installed here")

PATTERNS = sorted(name for name, flags in _core.FLAGS.items() if "pattern" in flags)

# Prices that move the way patterns need: gaps, doji bars, marubozu stretches
# and runs in one direction. Each row is (gap, drift, sigma, wick, flat), where
# `flat` is one bar in N closing where it opened and a zero wick leaves none.
REGIMES = [
    (0.0, 0.0, 0.010, 0.0035, 0),
    (0.0, 0.0, 0.010, 0.0, 0),
    (0.006, 0.0, 0.008, 0.0010, 0),
    (0.0, 0.012, 0.004, 0.0008, 0),
    (0.0, -0.012, 0.004, 0.0008, 0),
    (0.004, -0.010, 0.005, 0.0, 0),
    (0.004, 0.010, 0.005, 0.0, 0),
    (0.0, 0.0, 0.0015, 0.006, 3),
    (0.008, 0.0, 0.002, 0.004, 4),
    (0.0, 0.0, 0.012, 0.012, 0),
]
RUN = 45
RANDOM_SERIES = 24
RANDOM_BARS = 750


def alias_of(repo_root, name):
    import yaml

    path = repo_root / "crates" / "trendlib" / "src" / "indicators" / name / "spec.yaml"
    return yaml.safe_load(path.read_text(encoding="utf-8"))["talib"]["name"]


def read_golden(path):
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(line for line in handle if not line.startswith("#")))
    return {key: np.array([float(row[key]) for row in rows]) for key in rows[0]}


def random_bars(rng, count):
    close = 100.0
    rows = []
    for index in range(count):
        gap, drift, sigma, wick, flat = REGIMES[(index // RUN) % len(REGIMES)]
        open_ = close * (1.0 + gap * rng.normal()) if gap else close
        close = open_ * (1.0 + drift + sigma * rng.normal())
        if flat and rng.integers(flat) == 0:
            close = open_
        spread = abs(close - open_) * wick * 100.0 if wick else 0.0
        high = max(open_, close) + abs(rng.normal(0.0, spread + 1e-12))
        low = min(open_, close) - abs(rng.normal(0.0, spread + 1e-12))
        rows.append((open_, high, low, close))
    bars = np.array(rows, dtype=np.float64)
    return [bars[:, index].copy() for index in range(4)]


@pytest.mark.parametrize("name", PATTERNS)
def test_the_pattern_golden_holds_bars_the_pattern_fires_on(name, repo_root):
    """A golden of nothing but zeros tests only that the pattern stayed quiet.

    Three of the nine rules this suite caught were wrong in exactly that blind
    spot, so an empty golden is treated as a missing test rather than a pass.
    """
    folder = repo_root / "crates" / "trendlib" / "src" / "indicators" / name
    columns = read_golden(folder / "golden" / "patterns.csv")
    fired = np.flatnonzero(columns[name])
    assert fired.size, f"{name}: the patterns golden never fires, so it proves nothing"

    bars = [columns[field] for field in ("open", "high", "low", "close")]
    assert np.array_equal(np.asarray(getattr(tl, name)(*bars)), columns[name])


@pytest.mark.parametrize("seed", range(RANDOM_SERIES))
def test_every_pattern_agrees_with_talib_on_random_bars(seed, repo_root):
    """Random bars reach shapes no committed dataset was built to hold."""
    bars = random_bars(np.random.default_rng(20261008 + seed), RANDOM_BARS)
    for name in PATTERNS:
        mine = np.asarray(getattr(tl, name)(*bars))
        theirs = getattr(talib, alias_of(repo_root, name))(*bars)
        differing = np.flatnonzero(mine != theirs)
        assert not differing.size, (
            f"{name}: row {differing[0]} reads {mine[differing[0]]}, "
            f"the oracle reads {theirs[differing[0]]}"
        )
