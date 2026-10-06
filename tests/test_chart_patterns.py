"""Every chart pattern, on bars where it actually fires, and against its oracle.

The daily walk sits near a price of 1,000, where a slope of 0.02 a bar is about
as flat as a line can be drawn, so on it the triangles, rectangles and
broadening formation never fire at all and the flags and pennants barely do.
The `charts` case reads bars built so every pattern fires in each direction it
reads (`scripts/testdata/make_synthetic.py`); the tests here assert that it
does, and that TrendLib agrees with ta-patterns (oracle P) on random bars too.
"""

import csv
import functools
import re

import numpy as np
import pytest

import trendlib as tl
from trendlib import _core

CHARTS = sorted(name for name, flags in _core.FLAGS.items() if "chart_pattern" in flags)

RANDOM_SERIES = 12
RANDOM_BARS = 900


def golden_path(repo_root, name, case="charts"):
    return (
        repo_root / "crates" / "trendlib" / "src" / "indicators" / name / "golden" / f"{case}.csv"
    )


def read_golden(path):
    """The bars, the expected values and the oracle functions named in the header.

    The functions are read from the file rather than from `spec.yaml` so this
    module needs no YAML parser.
    """
    header, body = [], []
    with path.open(newline="", encoding="utf-8") as handle:
        for line in handle:
            (header if line.startswith("#") else body).append(line)
    oracle = next(line for line in header if line.startswith("# oracle:"))
    rows = list(csv.DictReader(body))
    columns = {key: np.array([float(row[key]) for row in rows]) for key in rows[0]}
    return columns, re.findall(r"chart_patterns\.([a-z_]+)", oracle)


@functools.cache
def directions(repo_root):
    """What each pattern reads, from the `Reads` column of INDICATORS.md § 5.1."""
    text = (repo_root / "docs" / "INDICATORS.md").read_text(encoding="utf-8")
    section = text.split("### 5.1 Chart patterns", 1)[1]
    found = {}
    for name, reads in re.findall(
        r"^\| `(chart_[a-z0-9_]+)` \|[^|]+\|[^|]+\| ([^|]+) \|", section, re.M
    ):
        reads = reads.strip()
        found[name] = {"+100": {100}, "-100": {-100}, "±100": {100, -100}}[reads]
    return found


def test_every_approved_chart_pattern_is_built(repo_root):
    assert set(CHARTS) == set(directions(repo_root))


@pytest.mark.parametrize("name", CHARTS)
def test_the_charts_golden_fires_in_every_direction_the_pattern_reads(name, repo_root):
    """A golden of nothing but zeros tests only that the pattern stayed quiet."""
    columns, _ = read_golden(golden_path(repo_root, name))
    fired = set(np.unique(columns[name][columns[name] != 0]).astype(int))
    assert fired == directions(repo_root)[name], f"{name}: the charts golden reads {fired}"

    bars = [columns[field] for field in ("high", "low", "close")]
    assert np.array_equal(np.asarray(getattr(tl, name)(*bars)), columns[name])


@pytest.mark.parametrize("name", CHARTS)
def test_a_chart_pattern_reads_only_plus_or_minus_one_hundred(name, bars):
    found = np.asarray(getattr(tl, name)(bars["high"], bars["low"], bars["close"]))
    assert found.dtype == np.int32
    assert set(np.unique(found)) <= {-100, 0, 100}


# ---------------------------------------------------------------------------
# Agreement with the oracle on bars no committed dataset holds
# ---------------------------------------------------------------------------

# Parameters TrendLib renames (D12).
RENAMED = {"period": "window"}


@pytest.fixture(scope="module")
def ta_patterns():
    """The oracle, where it is installed; the tests above need only the goldens."""
    return pytest.importorskip("ta_patterns.chart_patterns", reason="oracle P is not installed")


@functools.cache
def oracle_functions(repo_root):
    return {name: read_golden(golden_path(repo_root, name, "default"))[1] for name in CHARTS}


def oracle(ta_patterns, functions, high, low, close, params):
    keywords = {RENAMED.get(key, key): value for key, value in params.items()}
    readings = [
        getattr(ta_patterns, function)(
            close, high, low, close, mode="confirmed", **keywords
        ).astype(np.int64)
        * 100
        for function in functions
    ]
    if len(readings) == 1:
        return readings[0]
    return np.where(readings[0] != 0, readings[0], readings[1])


def random_bars(rng, count, scale):
    """A walk with a slow swing in it, so lines and poles form now and then."""
    steps = rng.normal(0.0, 0.01, count) + 0.003 * np.sin(np.arange(count) / rng.uniform(4, 30))
    close = scale * np.cumprod(1.0 + steps)
    spread = np.abs(rng.normal(0.0, 0.006, count)) * close
    high = close + spread * rng.uniform(0.0, 1.0, count)
    low = close - spread * rng.uniform(0.0, 1.0, count)
    return high, low, close


def compare(ta_patterns, name, functions, high, low, close, params):
    mine = np.asarray(getattr(tl, name)(high, low, close, **params))
    theirs = oracle(ta_patterns, functions, high, low, close, params)
    differing = np.flatnonzero(mine != theirs)
    if "pole_bars" in params:
        # Deviation 8: on the first row a pole could end on, the oracle reads
        # the last bar of the input. Dropping the row outright would hide a
        # regression that happens to land on it, so what TrendLib does there is
        # asserted instead: the row is warm-up and reads 0.
        row = params["pole_bars"] + params["period"]
        if row < mine.size:
            assert mine[row] == 0, (
                f"{name} {params}: row {row} is deviation 8's warm-up row and "
                f"should read 0, not {mine[row]}"
            )
            differing = differing[differing != row]
    assert not differing.size, (
        f"{name} {params}: row {differing[0]} reads {mine[differing[0]]}, "
        f"the oracle reads {theirs[differing[0]]}"
    )
    return int(np.count_nonzero(mine))


def defaults(name):
    return {param: spec["default"] for param, spec in _core.PARAMS[name].items()}


@pytest.mark.parametrize("seed", range(RANDOM_SERIES))
def test_every_chart_pattern_agrees_with_the_oracle_on_random_bars(seed, repo_root, ta_patterns):
    """Half the series sit near 1 and half near 100, where a flat line is a
    different slope, so every pattern gets bars it can fire on."""
    rng = np.random.default_rng(20261005 + seed)
    scale = 1.0 if seed % 2 else 100.0
    high, low, close = random_bars(rng, RANDOM_BARS, scale)
    for name, functions in oracle_functions(repo_root).items():
        compare(ta_patterns, name, functions, high, low, close, defaults(name))


@pytest.mark.parametrize("seed", range(RANDOM_SERIES))
def test_every_chart_pattern_agrees_with_the_oracle_at_random_parameters(
    seed, repo_root, ta_patterns
):
    rng = np.random.default_rng(20261105 + seed)
    high, low, close = random_bars(rng, RANDOM_BARS, 2.0 if seed % 2 else 50.0)
    ceilings = {"period": 120, "pivot_n": 8, "pole_bars": 20, "min_separation": 14}
    for name, functions in oracle_functions(repo_root).items():
        params = {}
        for param, spec in _core.PARAMS[name].items():
            if isinstance(spec["default"], int):
                params[param] = int(rng.integers(spec["min"], ceilings[param] + 1))
            else:
                params[param] = float(spec["default"] * rng.uniform(0.25, 3.0))
        compare(ta_patterns, name, functions, high, low, close, params)


def test_the_random_bars_reach_every_pattern(repo_root):
    """Agreement on bars where nothing fires would prove nothing."""
    fired = dict.fromkeys(CHARTS, 0)
    for seed in range(RANDOM_SERIES):
        rng = np.random.default_rng(20261005 + seed)
        high, low, close = random_bars(rng, RANDOM_BARS, 1.0 if seed % 2 else 100.0)
        for name in CHARTS:
            fired[name] += int(np.count_nonzero(getattr(tl, name)(high, low, close)))
    quiet = sorted(name for name, count in fired.items() if count == 0)
    assert not quiet, f"never fired on the random bars: {quiet}"
