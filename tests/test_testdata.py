"""The committed datasets are the inputs every golden file is computed from.

If one of them drifts, every golden value silently describes different bars, so
the contract in docs/TESTING.md section 3 is asserted here rather than trusted.
"""

import csv
import itertools
import subprocess
import sys
from datetime import datetime, timedelta, timezone

import pytest

IST = timezone(timedelta(hours=5, minutes=30))
NS_PER_SECOND = 1_000_000_000


def read_rows(path):
    with path.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle))


@pytest.fixture(scope="module")
def daily(testdata):
    return read_rows(testdata / "daily_2000.csv")


@pytest.fixture(scope="module")
def intraday(testdata):
    return read_rows(testdata / "intraday_5m_20d.csv")


@pytest.fixture(scope="module")
def patterns(testdata):
    return read_rows(testdata / "patterns_1080.csv")


def test_daily_shape(daily):
    assert len(daily) == 2000
    assert list(daily[0]) == ["date", "open", "high", "low", "close", "volume"]


def test_daily_bars_are_well_formed(daily):
    previous = ""
    for row in daily:
        o, h, low, c = (float(row[k]) for k in ("open", "high", "low", "close"))
        assert h >= max(o, c)
        assert low <= min(o, c)
        assert low > 0.0
        assert int(row["volume"]) > 0
        assert row["date"] > previous
        previous = row["date"]


def test_daily_walks_away_from_its_start(daily):
    assert 100.0 < float(daily[0]["close"]) < 10_000.0
    assert float(daily[-1]["close"]) != float(daily[0]["close"])


def test_intraday_shape(intraday):
    assert len(intraday) == 20 * 75
    assert list(intraday[0]) == ["timestamp", "open", "high", "low", "close", "volume"]


def test_intraday_sessions_run_0915_to_1525_ist(intraday):
    for index, row in enumerate(intraday):
        moment = datetime.fromtimestamp(int(row["timestamp"]) / NS_PER_SECOND, IST)
        if index % 75 == 0:
            assert (moment.hour, moment.minute) == (9, 15)
        if index % 75 == 74:
            assert (moment.hour, moment.minute) == (15, 25)


def test_intraday_timestamps_strictly_increase(intraday):
    stamps = [int(row["timestamp"]) for row in intraday]
    assert all(b > a for a, b in itertools.pairwise(stamps))


def test_intraday_covers_twenty_distinct_sessions_with_a_weekend_gap(intraday):
    dates = sorted(
        {
            datetime.fromtimestamp(int(row["timestamp"]) / NS_PER_SECOND, IST).date()
            for row in intraday
        }
    )
    assert len(dates) == 20
    gaps = {b - a for a, b in itertools.pairwise(dates)}
    assert timedelta(days=3) in gaps


def test_intraday_has_the_two_zero_volume_bars(intraday):
    zeros = [i for i, row in enumerate(intraday) if int(row["volume"]) == 0]
    # Session 3 opens with one (VWAP deviation 2), and session 7 has one in the
    # middle (a zero-volume bar must leave an anchored VWAP unchanged).
    assert zeros == [2 * 75, 6 * 75 + 40]
    assert all(int(row["volume"]) >= 0 for row in intraday)


def test_intraday_bars_are_well_formed(intraday):
    for row in intraday:
        o, h, low, c = (float(row[k]) for k in ("open", "high", "low", "close"))
        assert h >= max(o, c)
        assert low <= min(o, c)
        assert low > 0.0


def test_patterns_shape(patterns):
    assert len(patterns) == 1080
    assert list(patterns[0]) == ["date", "open", "high", "low", "close", "volume"]


def test_pattern_bars_are_well_formed(patterns):
    previous = ""
    for row in patterns:
        o, h, low, c = (float(row[k]) for k in ("open", "high", "low", "close"))
        assert h >= max(o, c)
        assert low <= min(o, c)
        assert low > 0.0
        assert int(row["volume"]) > 0
        assert row["date"] > previous
        previous = row["date"]


def test_generator_reproduces_the_committed_files(repo_root):
    result = subprocess.run(
        [sys.executable, "scripts/testdata/make_synthetic.py", "--check"],
        cwd=repo_root,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
