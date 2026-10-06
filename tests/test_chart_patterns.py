"""Every chart, bar and harmonic pattern, on bars where it fires, and against its oracle.

The daily walk sits near a price of 1,000, where a slope of 0.02 a bar is about
as flat as a line can be drawn, so on it the triangles, rectangles and
broadening formation never fire at all and the flags and pennants barely do.
The `charts` case reads bars built so those patterns fire in each direction
they read (`scripts/testdata/make_synthetic.py`). The tests here assert that
every pattern's golden files, between them, fire in every direction it reads,
and that TrendLib agrees with ta-patterns (oracle P) on random bars too.
"""

import csv
import functools
import inspect
import re

import numpy as np
import pytest

import trendlib as tl
from trendlib import _core

PATTERNS = sorted(name for name, flags in _core.FLAGS.items() if "chart_pattern" in flags)

RANDOM_SERIES = 12
RANDOM_BARS = 900

# The four oracle functions that read the input's last bar on the first row a
# pole can end on (CONVENTIONS.md deviation 8).
LOOKAHEAD_POLE = {"flag_bull", "flag_bear", "pennant_bull", "pennant_bear"}


def golden_dir(repo_root, name):
    return repo_root / "crates" / "trendlib" / "src" / "indicators" / name / "golden"


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
    return columns, re.findall(r"chart_patterns\.([a-z_0-9]+)", oracle)


def golden_params(path, name):
    """The parameters a golden file was produced with, typed as the spec types them."""
    with path.open(encoding="utf-8") as handle:
        line = next(line for line in handle if line.startswith("# params:"))
    rendered = line.split(":", 1)[1].strip()
    if rendered == "none":
        return {}
    params = {}
    for field in rendered.split(", "):
        key, value = field.split("=")
        kind = type(_core.PARAMS[name][key]["default"])
        params[key] = kind(float(value)) if kind is int else float(value)
    return params


READS = {"+100": {100}, "-100": {-100}, "±100": {100, -100}, "+100 (shape)": {100}}


@functools.cache
def directions(repo_root):
    """What each pattern reads, from the `Reads` column of INDICATORS.md §§ 5.1 and 5.2."""
    text = (repo_root / "docs" / "INDICATORS.md").read_text(encoding="utf-8")
    section = text.split("### 5.1 Chart patterns", 1)[1].split("### 5.3", 1)[0]
    found = {}
    for name, reads in re.findall(
        r"^\| `((?:chart|bar|harmonic)_[a-z0-9_]+)` \|[^|]+\|[^|]+\| ([^|]+) \|", section, re.M
    ):
        found[name] = READS[reads.strip()]
    return found


def columns_for(name, bars):
    return [bars[field] for field in _core.INPUTS[name]]


def built(repo_root):
    """The approved patterns whose folder exists. One is unticked in
    docs/PROGRESS.md until it does; nothing here asserts on its absence."""
    return {name: reads for name, reads in directions(repo_root).items() if name in PATTERNS}


def test_every_built_pattern_is_approved(repo_root):
    assert set(PATTERNS) <= set(directions(repo_root))


@pytest.mark.parametrize("name", PATTERNS)
def test_the_goldens_fire_in_every_direction_the_pattern_reads(name, repo_root):
    """A golden of nothing but zeros tests only that the pattern stayed quiet."""
    fired = set()
    for path in sorted(golden_dir(repo_root, name).glob("*.csv")):
        columns, _ = read_golden(path)
        fired |= set(np.unique(columns[name][columns[name] != 0]).astype(int))
        found = getattr(tl, name)(*columns_for(name, columns), **golden_params(path, name))
        assert np.array_equal(np.asarray(found), columns[name]), (
            f"{path.name}: the Python call disagrees with the golden"
        )
    assert fired == built(repo_root)[name], f"{name}: the goldens read {fired}"


@pytest.mark.parametrize("name", PATTERNS)
def test_a_pattern_reads_only_plus_or_minus_one_hundred(name, bars):
    found = np.asarray(getattr(tl, name)(*columns_for(name, bars)))
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
    return {name: read_golden(golden_dir(repo_root, name) / "default.csv")[1] for name in PATTERNS}


def oracle(ta_patterns, functions, bars, params):
    keywords = {RENAMED.get(key, key): value for key, value in params.items()}
    readings = []
    for function in functions:
        detector = getattr(ta_patterns, function)
        mode = {"mode": "confirmed"} if "mode" in inspect.signature(detector).parameters else {}
        found = detector(bars["open"], bars["high"], bars["low"], bars["close"], **mode, **keywords)
        readings.append(found.astype(np.int64) * 100)
    if len(readings) == 1:
        return readings[0]
    return np.where(readings[0] != 0, readings[0], readings[1])


def random_bars(rng, count, scale):
    """A walk with a slow swing in it, so lines and poles form now and then."""
    steps = rng.normal(0.0, 0.01, count) + 0.003 * np.sin(np.arange(count) / rng.uniform(4, 30))
    close = scale * np.cumprod(1.0 + steps)
    opened = np.concatenate([[close[0]], close[:-1]]) * (1.0 + rng.normal(0.0, 0.004, count))
    spread = np.abs(rng.normal(0.0, 0.006, count)) * close
    return {
        "open": opened,
        "high": np.maximum(opened, close) + spread * rng.uniform(0.0, 1.0, count),
        "low": np.minimum(opened, close) - spread * rng.uniform(0.0, 1.0, count),
        "close": close,
    }


def compare(ta_patterns, name, functions, bars, params):
    mine = np.asarray(getattr(tl, name)(*columns_for(name, bars), **params))
    theirs = oracle(ta_patterns, functions, bars, params)
    differing = np.flatnonzero(mine != theirs)
    if set(functions) & LOOKAHEAD_POLE:
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
def test_every_pattern_agrees_with_the_oracle_on_random_bars(seed, repo_root, ta_patterns):
    """Half the series sit near 1 and half near 100, where a flat line is a
    different slope, so every pattern gets bars it can fire on."""
    rng = np.random.default_rng(20261005 + seed)
    bars = random_bars(rng, RANDOM_BARS, 1.0 if seed % 2 else 100.0)
    for name, functions in oracle_functions(repo_root).items():
        compare(ta_patterns, name, functions, bars, defaults(name))


def random_params(rng, name):
    """Whole numbers between the minimum and a few times the default, so a case
    stays quick and still lands near the minimum; fractions scaled around the
    default."""
    params = {}
    for param, spec in _core.PARAMS[name].items():
        if isinstance(spec["default"], int):
            ceiling = min(spec["max"], max(3 * spec["default"], spec["min"] + 4), 150)
            params[param] = int(rng.integers(spec["min"], ceiling + 1))
        else:
            params[param] = float(spec["default"] * rng.uniform(0.25, 3.0))
    return params


@pytest.mark.parametrize("seed", range(RANDOM_SERIES))
def test_every_pattern_agrees_with_the_oracle_at_random_parameters(seed, repo_root, ta_patterns):
    rng = np.random.default_rng(20261105 + seed)
    bars = random_bars(rng, RANDOM_BARS, 2.0 if seed % 2 else 50.0)
    for name, functions in oracle_functions(repo_root).items():
        compare(ta_patterns, name, functions, bars, random_params(rng, name))


def test_the_random_bars_reach_every_pattern(repo_root):
    """Agreement on bars where nothing fires would prove nothing."""
    fired = dict.fromkeys(PATTERNS, 0)
    for seed in range(RANDOM_SERIES):
        rng = np.random.default_rng(20261005 + seed)
        bars = random_bars(rng, RANDOM_BARS, 1.0 if seed % 2 else 100.0)
        for name in PATTERNS:
            fired[name] += int(np.count_nonzero(getattr(tl, name)(*columns_for(name, bars))))
    quiet = sorted(name for name, count in fired.items() if count == 0)
    assert not quiet, f"never fired on the random bars: {quiet}"
