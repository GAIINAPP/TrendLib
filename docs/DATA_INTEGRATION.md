# Feeding TrendLib from a database

TrendLib computes indicators over bars you already have in memory. It does not
open connections, and `SPEC.md` § 3 keeps data feeds and broker connectors out
of scope on purpose: the core crate has zero runtime dependencies, and every
caller owns its own credentials, pooling, retries and caching.

That leaves exactly one thing between a SQL result set and an indicator: the
frame contract in § 2. Get that right and every function, stream and alias in
`PYTHON_API.md` works unchanged. This page is the contract, the loader patterns
that satisfy it, and the mistakes that produce confidently wrong numbers.

Nothing here is specific to one vendor or one schema. Map your columns to the
names in § 2 and the rest follows.

## 1. The boundary

```
  your database                     your loader                    TrendLib
┌────────────────┐            ┌──────────────────────┐        ┌──────────────┐
│ bars table(s)  │ ──SQL──►   │ rows → DataFrame     │ ──►    │ tl.rsi(df)   │
│ credentials    │            │ one instrument/frame │        │ tl.stream.*  │
│ pooling, cache │            │ tz-aware, sorted     │        │              │
└────────────────┘            └──────────────────────┘        └──────────────┘
      yours                     yours, ~20 lines                 this library
```

The loader is small and it is yours. TrendLib never sees a DSN, a driver or a
table name, so a schema change is a change in one function of your code and
never a library upgrade.

## 2. The frame contract

A valid input frame is a pandas or polars DataFrame holding **one instrument,
one interval**, with these columns. Column matching is case-insensitive
(`PYTHON_API.md` § 1).

| Column | Type | Required for | Rule |
| --- | --- | --- | --- |
| `open` | float64 | bar indicators | finite after the first valid bar |
| `high` | float64 | bar indicators | `high >= low` on every row |
| `low` | float64 | bar indicators | |
| `close` | float64 | everything | single-series indicators use this column |
| `volume` | float64 | volume indicators | `0` is valid, negative is not |
| timestamps | tz-aware datetime64 | session-anchored indicators | see below |

Timestamps come from a pandas `DatetimeIndex`, or from a column named
`timestamp`, `datetime`, `date` or `time` when the index is not datetime-like.
They must be timezone-aware, or int64 epoch nanoseconds read as UTC (`D10`).
A naive datetime raises, because anchoring a session without a zone is
meaningless.

Row rules:

- **Ascending by time, no duplicates.** TrendLib does not sort or dedupe. An
  out-of-order row is treated as the next bar and quietly corrupts every
  path-dependent indicator (Wilder smoothing, Supertrend, OBV, anchored VWAP).
- **Leading NaNs are skipped; a NaN or ±inf after the first valid bar raises**
  (`D8`). This is deliberate: a loud error beats a plausible wrong number.
- **Equal length** across all inputs when you pass arrays rather than a frame.
- Gaps are fine. Weekends, holidays and halts need no filler rows, and you
  should not invent them: a synthetic flat bar changes true range, ATR and
  every average that follows it.

### Mapping your columns

If your table uses other names, rename on the way in. Do it in the loader, once.

```python
BAR_COLUMNS = {
    "ts":  "timestamp",     # your name -> contract name
    "o":   "open",
    "h":   "high",
    "l":   "low",
    "c":   "close",
    "vol": "volume",
}

df = df.rename(columns=BAR_COLUMNS)
```

## 3. One instrument per frame

This is the rule most likely to be broken by a SQL result set, and it fails
silently.

- **A multi-symbol query returns many instruments.** Group before computing;
  never pass the raw result.
- **A dual-listed symbol is two instruments.** The same ticker on two exchanges
  has two price series. Concatenating them interleaves bars by timestamp and
  produces numbers that look plausible and mean nothing. Filter to one exchange,
  or treat `(symbol, exchange)` as the key.
- **A multi-interval table holds many series.** Filter to exactly one interval.
  Mixing 5-minute and 60-minute rows is the same bug wearing a different hat.

```python
for (symbol, exchange), bars in rows.groupby(["symbol", "exchange"], sort=False):
    bars = bars.set_index("timestamp").sort_index()
    out[(symbol, exchange)] = tl.rsi(bars["close"], period=14)
```

If your bars table has an exchange or interval column, the safe habit is to make
every key column part of the `groupby`, so adding a dimension later cannot
silently merge series.

## 4. Daily bars

A daily table is usually keyed by `(symbol, date)` with a SQL `DATE` column.
`DATE` reads back as a **naive** timestamp, so it must be localised before any
session-anchored indicator will accept it.

```sql
SELECT date, open, high, low, close, volume
FROM   bars_daily
WHERE  symbol = %(symbol)s
  AND  date >= %(start)s
ORDER  BY date ASC
```

```python
import os, pandas as pd, trendlib as tl
from sqlalchemy import create_engine

engine = create_engine(os.environ["BARS_DATABASE_URL"])   # never hard-code a DSN

def load_daily(symbol: str, start: str, tz: str = "Asia/Kolkata") -> pd.DataFrame:
    df = pd.read_sql(SQL_DAILY, engine, params={"symbol": symbol, "start": start})
    df["date"] = pd.to_datetime(df["date"]).dt.tz_localize(tz)   # DATE is naive
    df = df.set_index("date").sort_index()
    df = df[~df.index.duplicated(keep="last")]
    return df.astype({c: "float64" for c in ("open", "high", "low", "close", "volume")})

df = load_daily("EXAMPLE", "2024-01-01")
df["rsi"] = tl.rsi(df["close"], period=14)
df = df.join(tl.bbands(df, period=20))
```

Daily bars carry no intraday session, so `tz` here only has to be the zone the
dates were recorded in. It matters for multi-day anchors and for the trading
calendar work in `SPEC.md` F13.

## 5. Intraday bars

An intraday table is usually keyed by `(symbol, exchange, interval, timestamp)`
with a `TIMESTAMPTZ` column. Filter to one symbol, one exchange and one
interval, and keep the zone.

```sql
SELECT timestamp, open, high, low, close, volume
FROM   bars_intraday
WHERE  symbol   = %(symbol)s
  AND  exchange = %(exchange)s
  AND  interval = %(interval)s
  AND  timestamp >= %(start)s
ORDER  BY timestamp ASC
```

```python
def load_intraday(symbol, exchange, interval, start, tz="Asia/Kolkata"):
    df = pd.read_sql(SQL_INTRADAY, engine, params=locals())
    ts = pd.to_datetime(df.pop("timestamp"), utc=True).dt.tz_convert(tz)
    df = df.set_index(ts).sort_index()
    return df[~df.index.duplicated(keep="last")].astype("float64")

bars = load_intraday("EXAMPLE", "NSE", "5m", "2026-01-01")
bars["vwap"] = tl.vwap(bars, anchor="day", tz="Asia/Kolkata")
```

Read a `TIMESTAMPTZ` as UTC and convert, rather than localising it. Localising
an already-aware timestamp is either an error or a silent shift, and a shifted
timestamp moves every session boundary, which is exactly what `anchor="day"`
depends on.

## 6. Continuing a live feed from stored history

This is the pattern that makes a dashboard agree with a backtest. Open a stream
on history from the database, then feed it live bars. The parity guarantee in
`PYTHON_API.md` § 4 means the streamed values are bitwise identical to running
the batch function over the whole extended series, so a number on a live screen
and the same number recomputed later cannot disagree.

```python
history = load_intraday("EXAMPLE", "NSE", "5m", start_of_day)

s = tl.stream.rsi(history["close"], period=14)    # warm up from the database
live = s.peek(forming_close)                      # value if this bar closed now
final = s.update(closed_close)                    # commit the closed bar
```

Rules worth holding onto:

- `update` commits a **closed** bar. `peek` evaluates a forming one and changes
  nothing, so it is the right call for a ticking candle.
- Open with at least `tl.lookback(name, **params) + 1` bars, otherwise you get
  `InsufficientHistory` telling you how many are needed.
- Persist nothing but the bars. A stream is a value, not a session: on restart,
  reload history and reopen. There is no global state to restore (`D9`).
- One stream per `(instrument, interval, parameter set)`. Changing a parameter
  means a new stream.

## 7. A preflight worth copying

Most integration bugs are contract violations that produce numbers instead of
errors. Checking at the loader boundary costs microseconds.

```python
def check_bars(df: pd.DataFrame) -> None:
    if not isinstance(df.index, pd.DatetimeIndex) or df.index.tz is None:
        raise ValueError("bars need a timezone-aware DatetimeIndex")
    if not df.index.is_monotonic_increasing:
        raise ValueError("bars must be sorted ascending by time")
    if df.index.has_duplicates:
        raise ValueError("duplicate timestamps: is more than one series mixed in?")
    missing = {"open", "high", "low", "close"} - set(df.columns.str.lower())
    if missing:
        raise ValueError(f"missing columns: {sorted(missing)}")
    if (df["high"] < df["low"]).any():
        raise ValueError("high < low on at least one bar")
```

## 8. Pitfalls

| Symptom | Cause | Fix |
| --- | --- | --- |
| `InvalidInput: naive datetime` | SQL `DATE`, or `TIMESTAMP` without a zone | localise to the recording zone (§ 4) |
| Session VWAP or CPR resets at the wrong bar | timestamps converted to the wrong zone | read `TIMESTAMPTZ` as UTC, then convert (§ 5) |
| Values drift from a broker chart on dual-listed names | two exchanges merged into one series | key on `(symbol, exchange)` (§ 3) |
| Indicators look noisy or wrong after a schema change | two intervals in one frame | filter to one interval (§ 3) |
| `InvalidInput` on a NaN mid-series | a gap row stored as NULL | drop the row, do not forward-fill a price |
| ATR or true range spikes at a holiday | synthetic flat bars inserted to fill gaps | store real bars only (§ 2) |
| Streamed value differs from a later recompute | history reloaded with a different interval, zone or exchange | load live and history through the same function (§ 6) |
| Results differ run to run | unsorted result set; SQL without `ORDER BY` is unordered | sort in SQL and again after loading |
| Precision differs slightly from another tool | float32 somewhere in the path | cast to float64 at the loader |

## 9. Related

- `PYTHON_API.md` § 1 for the DataFrame rules, § 4 for streaming.
- `CONVENTIONS.md` for warm-up length, NaN policy and timestamp handling.
- `SPEC.md` § 3 for why connectors stay out of the library, F5 for the frame
  requirement, F13 for the trading-calendar work that will cover holidays and
  session boundaries.
