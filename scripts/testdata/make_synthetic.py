"""Write the committed synthetic datasets in ``testdata/``.

Golden files are computed from these bars, so the datasets have to come out
identical on every machine, forever.

That rules out more than it sounds like. :class:`random.Random` is fine: its
Mersenne Twister stream and its ``random()`` and ``randrange()`` outputs are
part of CPython's documented behaviour and are built from integer arithmetic.
``random.gauss``, ``math.exp`` and ``math.log`` are not: they call into the
platform's libm, which is free to be a unit in the last place away from another
platform's, and these files are written with round-trip precision, so one ULP
is a different file. CI caught exactly that between Linux and macOS.

So everything below is built from operations IEEE-754 defines exactly:
addition, subtraction, multiplication, division and comparison. The normal-ish
variate is Irwin-Hall (twelve uniforms, minus six), the price walk compounds
``1 + mu + sigma * z`` instead of ``exp``, and the intraday volume curve is a
polynomial rather than a log-normal.

The committed CSVs remain the source of truth (``docs/TESTING.md`` section 3):
regenerate only on purpose, and regenerate every golden file in the same PR.

    python scripts/testdata/make_synthetic.py [--check]

``--check`` regenerates into memory and fails if the committed files differ.
"""

from __future__ import annotations

import argparse
import itertools
import random
import sys
from datetime import date, datetime, timedelta, timezone
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
TESTDATA = REPO_ROOT / "testdata"

DAILY_SEED = 20261004
INTRADAY_SEED = 20261005

IST = timezone(timedelta(hours=5, minutes=30))
NS_PER_SECOND = 1_000_000_000

DAILY_BARS = 2000
DAILY_START = date(2018, 1, 1)
DAILY_START_PRICE = 1000.0

SESSIONS = 20
BARS_PER_SESSION = 75
INTRADAY_START = date(2026, 1, 5)
INTRADAY_START_PRICE = 1000.0
SESSION_OPEN = (9, 15)
BAR_MINUTES = 5

# Hostile-but-legal rows the volume indicators have to cope with
# (docs/TESTING.md section 3). Sessions and bars are 1-based here.
EMPTY_OPEN_SESSION = 3
EMPTY_MID_SESSION = 7
EMPTY_MID_BAR = 41


def normalish(rng: random.Random) -> float:
    """A bell-shaped variate on [-6, 6], from addition alone.

    Twelve uniforms sum to a variate with mean 6 and variance 1 (Irwin-Hall),
    so subtracting 6 approximates a standard normal closely enough for test
    bars, without touching a transcendental function.
    """
    total = 0.0
    for _ in range(12):
        total += rng.random()
    return total - 6.0


def _bar(
    rng: random.Random, prev_close: float, gap: float, drift: float, sigma: float, wick: float
) -> tuple[float, float, float, float]:
    open_ = prev_close * (1.0 + gap * normalish(rng))
    close = open_ * (1.0 + drift + sigma * normalish(rng))
    high = max(open_, close) * (1.0 + wick * abs(normalish(rng)))
    low = min(open_, close) * (1.0 - wick * abs(normalish(rng)))
    return open_, high, low, close


def _weekdays(start: date, count: int) -> list[date]:
    days: list[date] = []
    day = start
    while len(days) < count:
        if day.weekday() < 5:
            days.append(day)
        day += timedelta(days=1)
    return days


def make_daily() -> str:
    rng = random.Random(DAILY_SEED)
    rows = ["date,open,high,low,close,volume"]
    close = DAILY_START_PRICE
    for day in _weekdays(DAILY_START, DAILY_BARS):
        open_, high, low, close = _bar(rng, close, 0.0015, 0.00025, 0.011, 0.004)
        volume = 200_000 + rng.randrange(800_000)
        rows.append(f"{day.isoformat()},{open_!r},{high!r},{low!r},{close!r},{volume}")
    return "\n".join(rows) + "\n"


def _intraday_volume(rng: random.Random, bar_index: int) -> int:
    # Real sessions trade heavily at the open and the close and quietly in the
    # middle; the shape matters because VWAP weights by volume. A cubic gives
    # that curve without an exponential.
    position = bar_index / (BARS_PER_SESSION - 1)
    rest = 1.0 - position
    shape = 1.0 + 2.2 * rest * rest * rest + 1.4 * position * position * position
    jitter = 0.7 + 0.6 * rng.random()
    return round(6_000.0 * shape * jitter)


def make_intraday() -> str:
    rng = random.Random(INTRADAY_SEED)
    rows = ["timestamp,open,high,low,close,volume"]
    close = INTRADAY_START_PRICE
    for session, day in enumerate(_weekdays(INTRADAY_START, SESSIONS), start=1):
        first = datetime(day.year, day.month, day.day, *SESSION_OPEN, tzinfo=IST)
        # The overnight gap is wider than any single 5-minute move.
        close *= 1.0 + 0.006 * normalish(rng)
        for bar in range(BARS_PER_SESSION):
            open_, high, low, close = _bar(rng, close, 0.0004, 0.0, 0.0019, 0.0009)
            volume = _intraday_volume(rng, bar)
            if session == EMPTY_OPEN_SESSION and bar == 0:
                volume = 0
            elif session == EMPTY_MID_SESSION and bar == EMPTY_MID_BAR - 1:
                volume = 0
            stamp = first + timedelta(minutes=BAR_MINUTES * bar)
            epoch_ns = int(stamp.timestamp()) * NS_PER_SECOND
            rows.append(f"{epoch_ns},{open_!r},{high!r},{low!r},{close!r},{volume}")
    return "\n".join(rows) + "\n"


def _check_daily(text: str) -> None:
    lines = text.splitlines()
    assert len(lines) == DAILY_BARS + 1, len(lines)
    previous_day = ""
    for line in lines[1:]:
        day, open_, high, low, close, volume = line.split(",")
        o, h, low_, c = float(open_), float(high), float(low), float(close)
        assert h >= max(o, c) and low_ <= min(o, c), line
        assert low_ > 0.0, line
        assert int(volume) > 0, line
        assert day > previous_day, line
        previous_day = day
    last = float(lines[-1].split(",")[4])
    assert 100.0 < last < 10_000.0, f"the walk left a plausible range: {last}"


def _check_intraday(text: str) -> None:
    lines = text.splitlines()
    assert len(lines) == SESSIONS * BARS_PER_SESSION + 1, len(lines)
    previous_ns = -1
    zero_volume_rows = []
    local_dates = set()
    for index, line in enumerate(lines[1:]):
        stamp, open_, high, low, close, volume = line.split(",")
        o, h, low_, c = float(open_), float(high), float(low), float(close)
        assert h >= max(o, c) and low_ <= min(o, c), line
        assert low_ > 0.0, line
        ns = int(stamp)
        assert ns > previous_ns, line
        previous_ns = ns
        moment = datetime.fromtimestamp(ns / NS_PER_SECOND, IST)
        local_dates.add(moment.date())
        if index % BARS_PER_SESSION == 0:
            assert (moment.hour, moment.minute) == SESSION_OPEN, line
        if index % BARS_PER_SESSION == BARS_PER_SESSION - 1:
            assert (moment.hour, moment.minute) == (15, 25), line
        if int(volume) == 0:
            zero_volume_rows.append(index)

    assert len(local_dates) == SESSIONS, sorted(local_dates)
    expected_zeros = [
        (EMPTY_OPEN_SESSION - 1) * BARS_PER_SESSION,
        (EMPTY_MID_SESSION - 1) * BARS_PER_SESSION + EMPTY_MID_BAR - 1,
    ]
    assert zero_volume_rows == expected_zeros, zero_volume_rows
    ordered = sorted(local_dates)
    gaps = [b - a for a, b in itertools.pairwise(ordered)]
    assert timedelta(days=3) in gaps, "no weekend gap in the intraday dataset"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail instead of writing if the committed files are out of date",
    )
    args = parser.parse_args()

    daily, intraday = make_daily(), make_intraday()
    _check_daily(daily)
    _check_intraday(intraday)

    failures = 0
    for name, text in (("daily_2000.csv", daily), ("intraday_5m_20d.csv", intraday)):
        path = TESTDATA / name
        if args.check:
            current = path.read_text(encoding="utf-8") if path.exists() else None
            if current != text:
                print(f"{path.relative_to(REPO_ROOT)} is out of date", file=sys.stderr)
                failures += 1
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8", newline="\n")
        print(f"wrote {path.relative_to(REPO_ROOT)} ({text.count(chr(10))} lines)")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
