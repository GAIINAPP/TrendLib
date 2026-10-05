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


PATTERNS_FILE = "patterns_1080.csv"
PATTERNS_SEED = 20261007
PATTERNS_START = date(2020, 1, 1)
PATTERNS_BASE = 100.0
PATTERNS_VOLUME = 500_000
# Quiet bars between one shape and the next. Every candle setting averages at
# most ten bars, so twelve of them means a shape is measured against the
# spacer alone and never against the shape before it.
PATTERNS_SPACER = 12
PATTERNS_WALK_BARS = 600
PATTERNS_RUN = 45

# One hand-built shape per pattern that a random walk does not reach often
# enough to test. Writing the bars out is the only way to promise a pattern
# fires at all: thirty-one of the sixty-one fired fewer than five times in
# fifteen hundred random bars, and twelve of those never fired once, so their
# golden files held nothing but zeros and proved only that the pattern stayed
# silent (docs/TESTING.md section 3).
#
# Each list is (open, high, low, close), laid down after a spacer. The oracle
# decides what they mean: these bars are chosen so TA-Lib reports the pattern,
# and TA-Lib is what the golden file records.
PATTERNS_SHAPES: dict[str, list[tuple[float, float, float, float]]] = {
    # long white, a black bar gapping above it, a black bar closing inside it
    "2crows": [(100.0, 103.2, 99.8, 103.0), (105.0, 105.2, 104.4, 104.5),
               (104.8, 104.9, 101.3, 101.5)],
    # white, then three black bars each opening inside the last one's body
    "3blackcrows": [(100.0, 103.2, 99.8, 103.0), (103.0, 103.1, 100.95, 101.0),
                    (102.0, 102.1, 99.45, 99.5), (101.0, 101.1, 97.95, 98.0)],
    # a long body, a short one inside it, then a close out the other side
    "3inside": [(103.0, 103.1, 99.9, 100.0), (101.0, 101.5, 100.9, 101.4),
                (101.5, 104.1, 101.4, 104.0)],
    # three rising white bars, then one black bar that takes all three back
    "3linestrike": [(100.0, 101.1, 99.9, 101.0), (100.5, 102.1, 100.4, 102.0),
                    (101.5, 103.1, 101.4, 103.0), (103.5, 103.6, 98.9, 99.0)],
    # long black with a long tail, a smaller black inside it, a black marubozu
    "3starsinsouth": [(100.0, 100.0, 93.0, 97.0), (99.0, 99.0, 96.0, 98.0),
                      (98.5, 98.5, 98.0, 98.0)],
    # long white, a doji gapping clear of it, a black bar gapping clear again
    "abandonedbaby": [(100.0, 103.1, 99.9, 103.0), (104.0, 104.1, 103.5, 104.0),
                      (102.8, 103.0, 100.9, 101.0)],
    # long white, a gap up, two higher bars, then a black bar back into the gap
    "breakaway": [(100.0, 103.1, 99.9, 103.0), (104.0, 104.6, 103.9, 104.5),
                  (104.7, 105.3, 104.6, 105.2), (105.4, 106.1, 105.3, 106.0),
                  (106.0, 106.1, 103.4, 103.5)],
    # four black bars, the first two marubozu, the last swallowing the third
    "concealbabyswall": [(103.0, 103.0, 102.0, 102.0), (102.0, 102.0, 101.0, 101.0),
                         (100.5, 101.5, 99.9, 100.0), (101.0, 102.0, 99.0, 99.5)],
    # two long bars of opposite colour closing at the same price
    "counterattack": [(103.0, 103.1, 99.9, 100.0), (97.0, 100.1, 96.9, 100.02)],
    # long white, then a black bar opening above it and closing past its middle
    "darkcloudcover": [(100.0, 103.1, 99.9, 103.0), (103.5, 103.6, 100.9, 101.0)],
    # long white, a doji gapping above it, a black bar deep into the white body
    "eveningdojistar": [(100.0, 103.1, 99.9, 103.0), (104.0, 104.1, 103.9, 104.0),
                        (103.5, 103.6, 100.9, 101.0)],
    # the same with a short body in place of the doji
    "eveningstar": [(100.0, 103.1, 99.9, 103.0), (104.0, 104.4, 103.9, 104.3),
                    (103.5, 103.6, 100.9, 101.0)],
    # two white bars of the same size and open, both gapping above the first
    "gapsidesidewhite": [(100.0, 100.6, 99.4, 100.5), (101.0, 102.1, 100.9, 102.0),
                         (101.0, 102.0, 100.9, 101.9)],
    # three narrowing bars, the second closing at its high, then a higher bar
    "hikkakemod": [(100.0, 103.0, 97.0, 102.0), (98.5, 102.5, 98.0, 102.4),
                   (100.0, 102.0, 99.0, 101.0), (100.5, 103.0, 100.0, 102.5)],
    # long black, then a white bar closing just at the black bar's close
    "inneck": [(103.0, 103.1, 99.9, 100.0), (99.5, 100.1, 99.4, 100.02)],
    # a black marubozu, then a white marubozu gapping clear above it
    "kicking": [(103.0, 103.0, 100.0, 100.0), (104.0, 107.0, 104.0, 107.0)],
    # the same the other way round, so the longer body decides the direction
    "kickingbylength": [(100.0, 103.0, 100.0, 103.0), (99.0, 99.0, 96.0, 96.0)],
    # three falling black bars, a fourth with an upper shadow, then a white bar
    "ladderbottom": [(104.0, 104.1, 102.9, 103.0), (103.0, 103.1, 101.9, 102.0),
                     (102.0, 102.1, 100.9, 101.0), (101.0, 102.0, 100.4, 100.5),
                     (101.5, 103.1, 101.4, 103.0)],
    # long white, a gap up, three resting bars, then a white bar above them all
    "mathold": [(100.0, 103.1, 99.9, 103.0), (104.0, 104.1, 103.5, 103.6),
                (102.9, 103.0, 102.4, 102.5), (102.4, 102.5, 101.9, 102.0),
                (102.2, 105.1, 102.1, 105.0)],
    # long black, a doji gapping below it, a white bar deep into the black body
    "morningdojistar": [(103.0, 103.1, 99.9, 100.0), (99.0, 99.1, 98.9, 99.0),
                        (99.5, 102.1, 99.4, 102.0)],
    # the same with a short body in place of the doji
    "morningstar": [(103.0, 103.1, 99.9, 100.0), (99.0, 99.1, 98.6, 98.7),
                    (99.5, 102.1, 99.4, 102.0)],
    # long black, then a white bar closing back at the black bar's low
    "onneck": [(103.0, 103.1, 99.9, 100.0), (99.5, 100.0, 99.4, 99.92)],
    # long black, then a long white bar closing above the black bar's middle
    "piercing": [(103.0, 103.1, 99.9, 100.0), (99.5, 102.1, 99.4, 102.0)],
    # long white, three small black bars overlapping it, then a higher white
    "risefall3methods": [(100.0, 103.1, 99.9, 103.0), (102.5, 102.6, 102.1, 102.2),
                         (102.2, 102.3, 101.7, 101.8), (101.8, 101.9, 101.3, 101.4),
                         (101.6, 104.1, 101.5, 104.0)],
    # black, then a long white bar opening where the black one opened
    "separatinglines": [(102.0, 102.1, 99.9, 100.0), (102.0, 104.1, 101.95, 104.0)],
    # black, white, black, the two black bars closing at the same price
    "sticksandwich": [(102.0, 102.1, 99.9, 100.0), (100.6, 101.6, 100.5, 101.5),
                      (101.5, 101.6, 99.9, 100.02)],
    # a gap up, then a black bar of the same size closing back inside the gap
    "tasukigap": [(100.0, 100.6, 99.4, 100.5), (101.0, 102.1, 100.9, 102.0),
                  (101.5, 101.6, 100.55, 100.6)],
    # long black, then a white bar closing inside it but short of the middle
    "thrusting": [(103.0, 103.1, 99.9, 100.0), (99.5, 101.1, 99.4, 101.0)],
    # black, a smaller black reaching lower, then a short white bar
    "unique3river": [(103.0, 103.1, 99.9, 100.0), (102.0, 102.1, 99.5, 101.0),
                     (100.5, 101.1, 100.4, 101.0)],
    # long white, a short black gapping above it, a black bar swallowing that
    "upsidegap2crows": [(100.0, 103.1, 99.9, 103.0), (104.5, 104.6, 103.9, 104.0),
                        (105.0, 105.1, 103.4, 103.5)],
    # two white bars with a gap between them, then a black bar closing in it
    "xsidegap3methods": [(100.0, 101.1, 99.9, 101.0), (102.0, 103.1, 101.9, 103.0),
                         (102.5, 102.6, 100.4, 100.5)],
}

# The walk that follows the shapes, so the thirty patterns a random series does
# reach are tested on many bars rather than on one built for them. Each row is
# (gap, drift, sigma, wick, flat): `flat` is one bar in N closing where it
# opened, and a zero wick makes marubozu bars.
PATTERNS_REGIMES = [
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


def _pattern_spacer() -> list[tuple[float, float, float, float]]:
    """Bars with equal bodies, so no "long body" test can pass inside them."""
    base, body, wick = PATTERNS_BASE, 0.5, 0.25
    pair = [
        (base, base + body + wick, base - wick, base + body),
        (base + body, base + body + wick, base - wick, base),
    ]
    return [pair[index % 2] for index in range(PATTERNS_SPACER)]


def _pattern_walk(rng: random.Random, start: float) -> list[tuple[float, float, float, float]]:
    bars = []
    close = start
    for index in range(PATTERNS_WALK_BARS):
        gap, drift, sigma, wick, flat = PATTERNS_REGIMES[
            (index // PATTERNS_RUN) % len(PATTERNS_REGIMES)
        ]
        open_ = close * (1.0 + gap * normalish(rng))
        close = open_ * (1.0 + drift + sigma * normalish(rng))
        if flat and rng.randrange(flat) == 0:
            close = open_
        high = max(open_, close) * (1.0 + wick * abs(normalish(rng)))
        low = min(open_, close) * (1.0 - wick * abs(normalish(rng)))
        bars.append((open_, high, low, close))
    return bars


def make_patterns() -> str:
    rng = random.Random(PATTERNS_SEED)
    bars: list[tuple[float, float, float, float]] = []
    for shape in PATTERNS_SHAPES.values():
        bars.extend(_pattern_spacer())
        bars.extend(shape)
    bars.extend(_pattern_spacer())
    bars.extend(_pattern_walk(rng, PATTERNS_BASE))

    rows = ["date,open,high,low,close,volume"]
    for day, (open_, high, low, close) in zip(
        _weekdays(PATTERNS_START, len(bars)), bars, strict=True
    ):
        rows.append(
            f"{day.isoformat()},{open_!r},{high!r},{low!r},{close!r},{PATTERNS_VOLUME}"
        )
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


def _check_patterns(text: str) -> None:
    lines = text.splitlines()
    shaped = len(PATTERNS_SHAPES) * PATTERNS_SPACER + sum(
        len(shape) for shape in PATTERNS_SHAPES.values()
    )
    assert len(lines) == shaped + PATTERNS_SPACER + PATTERNS_WALK_BARS + 1, len(lines)
    previous_day = ""
    for line in lines[1:]:
        day, open_, high, low, close, volume = line.split(",")
        o, h, low_, c = float(open_), float(high), float(low), float(close)
        assert h >= max(o, c) and low_ <= min(o, c), line
        assert low_ > 0.0, line
        assert int(volume) > 0, line
        assert day > previous_day, line
        previous_day = day


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail instead of writing if the committed files are out of date",
    )
    args = parser.parse_args()

    daily, intraday, patterns = make_daily(), make_intraday(), make_patterns()
    _check_daily(daily)
    _check_intraday(intraday)
    _check_patterns(patterns)

    failures = 0
    for name, text in (
        ("daily_2000.csv", daily),
        ("intraday_5m_20d.csv", intraday),
        (PATTERNS_FILE, patterns),
    ):
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
