"""Produce golden CSVs for the oracle-A indicators of INDICATORS.md section 5.3.

    python scripts/oracle/formula_golden.py ichimoku                 # every case
    python scripts/oracle/formula_golden.py ichimoku --case default  # one case

No runnable library implements these, so the owner chose oracle A for them
(``docs/DECISIONS.md`` D20, ``docs/TESTING.md`` section 2): the published
formula the section 5.3 row cites, evaluated through TA-Lib's own functions
wherever one computes a step and NumPy for the rest. Every number a TA-Lib
function can produce comes from TA-Lib; only the shape of the expression is
written here, and each file's header names the functions that produced it.

This is the weaker oracle. It proves TrendLib computes the documented formula,
not that the formula is the one a given platform draws.
"""

from __future__ import annotations

import argparse
import csv
import datetime as dt
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
INDICATORS = REPO_ROOT / "crates" / "trendlib" / "src" / "indicators"
TESTDATA = REPO_ROOT / "testdata"

TOLERANCE = "rel=1e-10 abs=1e-12"
DAILY = "daily_2000.csv"

# Cases run at the defaults on another dataset, for a branch of the formula the
# daily walk never reaches. patterns_1080.csv has bars that close at their open
# and quiet bars that tie; locked_558.csv has bars with no range and no volume.
PATTERNS = "patterns_1080.csv"
LOCKED = "locked_558.csv"
OTHER_DATA = {
    "pivots_demark": {"patterns": PATTERNS},
    "connors_rsi": {"locked": LOCKED},
    "elder_impulse": {"locked": LOCKED},
    "twiggs_mf": {"locked": LOCKED},
    "rwi": {"locked": LOCKED},
    "safezone": {"locked": LOCKED},
    "wilder_volatility": {"locked": LOCKED},
    "swing_index": {"locked": LOCKED},
    "asi": {"locked": LOCKED},
    "frama": {"locked": LOCKED},
    "fractal_chaos_bands": {"patterns": PATTERNS, "locked": LOCKED},
    "darvas_box": {"patterns": PATTERNS, "locked": LOCKED},
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
    """`default`, and `min_period` with every whole-numbered parameter at its minimum."""
    found = {"default": defaults(spec)}
    floors = {
        param["name"]: param["min"]
        for param in spec.get("params") or []
        if param["type"] == "int" and "min" in param
    }
    if floors:
        found["min_period"] = {**defaults(spec), **floors}
    for case in OTHER_DATA.get(spec["name"], {}):
        found[case] = defaults(spec)
    return found


def dataset_for(name: str, case: str) -> str:
    return OTHER_DATA.get(name, {}).get(case, DAILY)


def read_columns(filename: str):
    import numpy as np

    with (TESTDATA / filename).open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    columns = {
        key: np.array([float(row[key]) for row in rows])
        for key in ("open", "high", "low", "close", "volume")
    }
    columns["source"] = columns["close"]
    return columns


def shifted(values, bars: int):
    """`values` drawn `bars` rows later: row t holds what row t - bars computed."""
    import numpy as np

    if bars == 0:
        return values.copy()
    out = np.full_like(values, np.nan)
    out[bars:] = values[:-bars]
    return out


def midpoint(high, low, period: int):
    """TA-Lib's MIDPRICE, or its MEDPRICE for the one-bar window MIDPRICE refuses."""
    import talib

    return (
        talib.MIDPRICE(high, low, timeperiod=period) if period >= 2 else talib.MEDPRICE(high, low)
    )


# Each formula returns its outputs in spec order, with the TA-Lib functions it
# used for the header. Rows before the indicator's warm-up are blanked by
# `warm_up`, which is TrendLib's alignment rule, not part of the formula.
def ichimoku(cols, p):
    import numpy as np
    import talib

    tenkan = midpoint(cols["high"], cols["low"], p["tenkan_period"])
    kijun = midpoint(cols["high"], cols["low"], p["kijun_period"])
    senkou = midpoint(cols["high"], cols["low"], p["senkou_period"])
    span_a = talib.MULT(talib.ADD(tenkan, kijun), np.full_like(tenkan, 0.5))
    d = p["displacement"]
    longest = max(p["tenkan_period"], p["kijun_period"], p["senkou_period"])
    outputs = [tenkan, kijun, shifted(span_a, d), shifted(senkou, d)]
    return outputs, longest - 1 + d, "talib.MIDPRICE (MEDPRICE for a one-bar window), ADD and MULT"


def constant(like, value: float):
    import numpy as np

    return np.full_like(like, value)


def smma(values, period: int):
    """Williams' smoothed average: TA-Lib's SMA of the first `period`, then each bar
    moves it `1 / period` of the way to the new value."""
    import numpy as np
    import talib

    out = talib.SMA(values, timeperiod=period)
    if period == 1:
        return out
    k = 1.0 / period
    start = int(np.flatnonzero(~np.isnan(out))[0])
    for t in range(start + 1, len(values)):
        out[t] = (values[t] - out[t - 1]) * k + out[t - 1]
    return out


def alligator_lines(cols, p):
    import talib

    median = talib.MEDPRICE(cols["high"], cols["low"])
    names = ("jaw", "teeth", "lips")
    lines = [shifted(smma(median, p[f"{n}_period"]), p[f"{n}_shift"]) for n in names]
    return lines, max(p[f"{n}_period"] - 1 + p[f"{n}_shift"] for n in names)


def alligator(cols, p):
    lines, lookback = alligator_lines(cols, p)
    return lines, lookback, "talib.MEDPRICE and SMA (the seed), the smoothing step in NumPy"


def gator(cols, p):
    import numpy as np
    import talib

    (jaw, teeth, lips), lookback = alligator_lines(cols, p)
    outputs = [np.abs(talib.SUB(jaw, teeth)), -np.abs(talib.SUB(teeth, lips))]
    return outputs, lookback, "talib.MEDPRICE, SMA (the seed) and SUB, the smoothing step in NumPy"


def envelope(cols, p):
    import talib

    middle = talib.SMA(cols["source"], timeperiod=p["period"])
    upper = talib.MULT(middle, constant(middle, 1.0 + p["percent"] / 100.0))
    lower = talib.MULT(middle, constant(middle, 1.0 - p["percent"] / 100.0))
    return [upper, middle, lower], p["period"] - 1, "talib.SMA and MULT"


GUPPY = (3, 5, 8, 10, 12, 15, 30, 35, 40, 45, 50, 60)


def guppy(cols, p):
    import talib

    lines = [talib.EMA(cols["source"], timeperiod=n) for n in GUPPY]
    return lines, max(GUPPY) - 1, "talib.EMA"


def woodies_cci(cols, p):
    import talib

    hlc = (cols["high"], cols["low"], cols["close"])
    outputs = [
        talib.CCI(*hlc, timeperiod=p["cci_period"]),
        talib.CCI(*hlc, timeperiod=p["turbo_period"]),
    ]
    return outputs, max(p["cci_period"], p["turbo_period"]) - 1, "talib.CCI"


def atr_bands(cols, p):
    import talib

    average = talib.ATR(cols["high"], cols["low"], cols["close"], timeperiod=p["period"])
    width = talib.MULT(average, constant(average, p["shift"]))
    outputs = [talib.ADD(cols["close"], width), talib.SUB(cols["close"], width)]
    return outputs, p["period"], "talib.ATR, MULT, ADD and SUB"


def pivots_woodie(cols, p):
    import talib

    high, low, close = (shifted(cols[k], 1) for k in ("high", "low", "close"))
    two = constant(close, 2.0)
    pivot = talib.DIV(talib.ADD(talib.ADD(high, low), talib.MULT(two, close)), constant(close, 4.0))
    span = talib.SUB(high, low)
    outputs = [
        pivot,
        talib.SUB(talib.MULT(two, pivot), low),
        talib.SUB(talib.MULT(two, pivot), high),
        talib.ADD(pivot, span),
        talib.SUB(pivot, span),
    ]
    return outputs, 1, "talib.ADD, SUB, MULT and DIV"


def pivots_demark(cols, p):
    import numpy as np

    opened, high, low, close = (shifted(cols[k], 1) for k in ("open", "high", "low", "close"))
    total = np.where(
        close < opened,
        high + 2.0 * low + close,
        np.where(close > opened, 2.0 * high + low + close, high + low + 2.0 * close),
    )
    outputs = [total / 4.0, total / 2.0 - low, total / 2.0 - high]
    return outputs, 1, "NumPy arithmetic"


def one_bar_change(close):
    import talib

    return talib.ROC(close, timeperiod=1)


def connors_rsi(cols, p):
    import numpy as np
    import talib

    close = cols["source"]
    streak = np.full_like(close, np.nan)
    run = 0.0
    for t in range(1, len(close)):
        if close[t] > close[t - 1]:
            run = max(run, 0.0) + 1.0
        elif close[t] < close[t - 1]:
            run = min(run, 0.0) - 1.0
        else:
            run = 0.0
        streak[t] = run
    change = one_bar_change(close)
    m = p["rank_period"]
    ranked = np.full_like(close, np.nan)
    for t in range(1 + m, len(close)):
        ranked[t] = 100.0 * np.count_nonzero(change[t - m : t] < change[t]) / m
    price = talib.RSI(close, timeperiod=p["rsi_period"])
    streaked = talib.RSI(streak, timeperiod=p["streak_period"])
    lookback = max(p["rsi_period"], 1 + p["streak_period"], 1 + m)
    return (
        [(price + streaked + ranked) / 3.0],
        lookback,
        "talib.RSI and ROC, the streak and the percent rank counted in NumPy",
    )


def pmo(cols, p):
    import talib

    first = talib.EMA(one_bar_change(cols["source"]), timeperiod=p["first_period"] - 1)
    scaled = talib.MULT(first, constant(first, 10.0))
    line = talib.EMA(scaled, timeperiod=p["second_period"] - 1)
    signal = talib.EMA(line, timeperiod=p["signal_period"])
    lookback = p["first_period"] + p["second_period"] + p["signal_period"] - 4
    return [line, signal], lookback, "talib.ROC, EMA and MULT"


def elder_impulse(cols, p):
    import numpy as np
    import talib

    close = cols["source"]
    trend = talib.EMA(close, timeperiod=p["ema_period"])
    _, _, histogram = talib.MACD(
        close,
        fastperiod=p["fast_period"],
        slowperiod=p["slow_period"],
        signalperiod=p["signal_period"],
    )
    rising = (trend[1:] > trend[:-1]) & (histogram[1:] > histogram[:-1])
    falling = (trend[1:] < trend[:-1]) & (histogram[1:] < histogram[:-1])
    reading = np.full_like(close, np.nan)
    reading[1:] = np.where(rising, 1.0, np.where(falling, -1.0, 0.0))
    macd_lookback = max(p["fast_period"], p["slow_period"]) - 1 + p["signal_period"] - 1
    lookback = max(p["ema_period"] - 1, macd_lookback) + 1
    return [reading], lookback, "talib.EMA and MACD, the comparisons in NumPy"


def wilder(values, period: int):
    """Wilder's smoothing: TA-Lib's SMA of the first `period`, then
    `prev * (n - 1) / n + x / n`."""
    import numpy as np
    import talib

    out = talib.SMA(values, timeperiod=period)
    start = int(np.flatnonzero(~np.isnan(out))[0])
    for t in range(start + 1, len(values)):
        out[t] = out[t - 1] * (period - 1) / period + values[t] / period
    return out


def twiggs_mf(cols, p):
    import numpy as np

    high, low, close = cols["high"], cols["low"], cols["close"]
    before = shifted(close, 1)
    top = np.maximum(high, before)
    bottom = np.minimum(low, before)
    span = top - bottom
    safe = np.where(span == 0.0, 1.0, span)
    flow = np.where(span == 0.0, 0.0, ((close - bottom) - (top - close)) / safe * cols["volume"])
    volume = cols["volume"].copy()
    volume[0] = np.nan
    flow_average = wilder(flow, p["period"])
    volume_average = wilder(volume, p["period"])
    divisor = np.where(volume_average == 0.0, 1.0, volume_average)
    ratio = np.where(volume_average == 0.0, 0.0, flow_average / divisor)
    return [ratio], p["period"], "talib.SMA (the seeds), Wilder's step and the ratio in NumPy"


def gapo(cols, p):
    import talib

    n = p["period"]
    span = talib.SUB(talib.MAX(cols["high"], timeperiod=n), talib.MIN(cols["low"], timeperiod=n))
    scale = talib.LN(constant(span, float(n)))
    return [talib.DIV(talib.LN(span), scale)], n - 1, "talib.MAX, MIN, SUB, LN and DIV"


def rwi(cols, p):
    import math

    import numpy as np
    import talib

    n = p["period"]
    high, low = cols["high"], cols["low"]
    average = talib.ATR(high, low, cols["close"], timeperiod=n)
    scale = talib.MULT(average, constant(average, math.sqrt(n)))
    safe = np.where(average == 0.0, 1.0, scale)
    rise = talib.SUB(high, shifted(low, n))
    fall = talib.SUB(shifted(high, n), low)
    outputs = [np.where(average == 0.0, 0.0, moved / safe) for moved in (rise, fall)]
    return outputs, n, "talib.ATR, SUB and MULT, the division in NumPy"


def linreg_channel(cols, p):
    import numpy as np
    import talib

    source, n = cols["source"], p["period"]
    middle = talib.LINEARREG(source, timeperiod=n)
    slope = talib.LINEARREG_SLOPE(source, timeperiod=n)
    intercept = talib.LINEARREG_INTERCEPT(source, timeperiod=n)
    sigma = np.full_like(source, np.nan)
    steps = np.arange(n, dtype=np.float64)
    for t in range(n - 1, len(source)):
        residuals = source[t - n + 1 : t + 1] - (intercept[t] + slope[t] * steps)
        sigma[t] = np.sqrt(np.sum(residuals * residuals) / (n - 1))
    spread = talib.MULT(sigma, constant(sigma, p["deviation"]))
    outputs = [talib.ADD(middle, spread), middle, talib.SUB(middle, spread)]
    return (
        outputs,
        n - 1,
        "talib.LINEARREG, LINEARREG_SLOPE, LINEARREG_INTERCEPT, MULT, ADD and SUB, "
        "the residuals in NumPy",
    )


def window_extreme(values, period: int, highest: bool):
    """TA-Lib's MAX or MIN, or the series itself for the one-bar window they refuse."""
    import talib

    if period == 1:
        return values.copy()
    return (talib.MAX if highest else talib.MIN)(values, timeperiod=period)


def window_sum(values, period: int):
    import talib

    return values.copy() if period == 1 else talib.SUM(values, timeperiod=period)


def safezone(cols, p):
    import numpy as np

    high, low = cols["high"], cols["low"]
    n, k, hold = p["period"], p["coefficient"], p["hold"]

    def mean_penetration(depth):
        hits = np.where(depth > 0.0, 1.0, 0.0)
        hits[0] = np.nan
        total, count = window_sum(depth, n), window_sum(hits, n)
        mean = np.where(count > 0.0, total / np.where(count > 0.0, count, 1.0), 0.0)
        return np.where(np.isnan(count), np.nan, mean)

    was_high, was_low = shifted(high, 1), shifted(low, 1)
    down = np.where(low < was_low, was_low - low, 0.0)
    up = np.where(high > was_high, high - was_high, 0.0)
    down[0] = up[0] = np.nan
    raw_lower = shifted(low - k * mean_penetration(down), 1)
    raw_upper = shifted(high + k * mean_penetration(up), 1)
    outputs = [window_extreme(raw_lower, hold, True), window_extreme(raw_upper, hold, False)]
    return outputs, n + hold, "talib.SUM, MAX and MIN, the penetrations and their mean in NumPy"


def wilder_volatility(cols, p):
    import numpy as np
    import talib

    close = cols["close"]
    n = p["period"]
    average = talib.ATR(cols["high"], cols["low"], close, timeperiod=n)
    reach = talib.MULT(average, constant(average, p["multiplier"]))
    level = np.full_like(close, np.nan)
    side = np.full_like(close, np.nan)
    rising = close[n] >= close[0]
    extreme = close[n]
    for t in range(n, len(close)):
        if t > n:
            if rising and close[t] < level[t - 1]:
                rising, extreme = False, close[t]
            elif not rising and close[t] > level[t - 1]:
                rising, extreme = True, close[t]
            else:
                extreme = max(extreme, close[t]) if rising else min(extreme, close[t])
        level[t] = extreme - reach[t] if rising else extreme + reach[t]
        side[t] = 1.0 if rising else -1.0
    return (
        [level, side],
        n,
        "talib.ATR and MULT, the swing followed bar by bar in NumPy (a transcription of the rule)",
    )


def swing_index_values(cols, limit: float):
    import numpy as np

    o, h, low, c = cols["open"], cols["high"], cols["low"], cols["close"]
    was_open, was_close = shifted(o, 1), shifted(c, 1)
    up = np.abs(h - was_close)
    down = np.abs(low - was_close)
    span = np.abs(h - low)
    body = np.abs(was_close - was_open)
    reach = np.where(
        (up >= down) & (up >= span),
        up - 0.5 * down + 0.25 * body,
        np.where((down >= up) & (down >= span), down - 0.5 * up + 0.25 * body, span + 0.25 * body),
    )
    moved = (c - was_close) + 0.5 * (c - o) + 0.25 * (was_close - was_open)
    safe = np.where(reach == 0.0, 1.0, reach)
    with np.errstate(divide="ignore", invalid="ignore"):
        swing = 50.0 * (moved / safe) * (np.maximum(up, down) / limit)
    swing = np.where(reach == 0.0, 0.0, swing)
    swing[0] = np.nan
    return swing


def swing_index(cols, p):
    swing = swing_index_values(cols, p["limit_move"])
    return [swing], 1, "NumPy arithmetic"


def asi(cols, p):
    import numpy as np

    swing = swing_index_values(cols, p["limit_move"])
    total = np.full_like(swing, np.nan)
    total[1:] = np.cumsum(swing[1:])
    return (
        [total],
        1,
        "NumPy arithmetic and a running sum",
    )


def frama(cols, p):
    import numpy as np
    import talib

    high, low = cols["high"], cols["low"]
    n = p["period"]
    h = n // 2

    def span(hi, lo, k):
        return window_extreme(hi, k, True) - window_extreme(lo, k, False)

    n1 = span(high, low, h) / h
    n2 = span(shifted(high, h), shifted(low, h), h) / h
    n3 = span(high, low, n) / n
    price = talib.MEDPRICE(high, low)
    out = np.full_like(price, np.nan)
    dimension = 0.0
    for t in range(n - 1, len(price)):
        if n1[t] > 0.0 and n2[t] > 0.0 and n3[t] > 0.0:
            dimension = (np.log(n1[t] + n2[t]) - np.log(n3[t])) / np.log(2.0)
        alpha = min(max(np.exp(-4.6 * (dimension - 1.0)), 0.01), 1.0)
        out[t] = price[t] if t == n - 1 else out[t - 1] + alpha * (price[t] - out[t - 1])
    return [out], n - 1, "talib.MAX, MIN and MEDPRICE, the dimension and the step in NumPy"


def fractal_chaos_bands(cols, p):
    import numpy as np

    left, right = p["left_bars"], p["right_bars"]
    bands = []
    for values, highest in ((cols["high"], True), (cols["low"], False)):
        middle = shifted(values, right)
        before = shifted(window_extreme(values, left, highest), right + 1)
        after = window_extreme(values, right, highest)
        if highest:
            swing = (middle > before) & (middle > after)
        else:
            swing = (middle < before) & (middle < after)
        band = np.full_like(values, np.nan)
        held = np.nan
        for t in range(len(values)):
            if swing[t]:
                held = middle[t]
            band[t] = held
        bands.append(band)
    return (
        bands,
        left + right,
        "talib.MAX and MIN (the swing tests), the bands carried forward in NumPy",
    )


def darvas_box(cols, p):
    import numpy as np

    high, low = cols["high"], cols["low"]
    c = p["confirm_bars"]
    top_edge = np.full_like(high, np.nan)
    bottom_edge = np.full_like(high, np.nan)
    phase = "forming"
    top, bottom, bottom_bar = -np.inf, None, None
    shown = (np.nan, np.nan)
    for t in range(len(high)):
        if phase == "boxed":
            if high[t] > top or low[t] < bottom:
                phase, top, bottom, bottom_bar = "forming", high[t], None, None
        else:
            if high[t] > top:
                top, bottom, bottom_bar = high[t], None, None
            elif bottom is None or low[t] < bottom:
                bottom, bottom_bar = low[t], t
            if bottom is not None and t - bottom_bar >= c:
                phase = "boxed"
                shown = (top, bottom)
        top_edge[t], bottom_edge[t] = shown
    return (
        [top_edge, bottom_edge],
        c + 1,
        "the rule followed bar by bar in NumPy (a transcription)",
    )


FORMULAS = {
    "ichimoku": ichimoku,
    "alligator": alligator,
    "gator": gator,
    "envelope": envelope,
    "guppy": guppy,
    "woodies_cci": woodies_cci,
    "atr_bands": atr_bands,
    "pivots_woodie": pivots_woodie,
    "pivots_demark": pivots_demark,
    "connors_rsi": connors_rsi,
    "pmo": pmo,
    "elder_impulse": elder_impulse,
    "twiggs_mf": twiggs_mf,
    "gapo": gapo,
    "rwi": rwi,
    "linreg_channel": linreg_channel,
    "safezone": safezone,
    "wilder_volatility": wilder_volatility,
    "swing_index": swing_index,
    "asi": asi,
    "frama": frama,
    "fractal_chaos_bands": fractal_chaos_bands,
    "darvas_box": darvas_box,
}


def warm_up(outputs, lookback: int, dtypes: list[str]):
    """Every output is warm-up before `lookback`, as TrendLib aligns a multi-output row:
    `NaN`, or `0` in an `int32` column (CONVENTIONS.md section 2)."""
    import numpy as np

    blanked = []
    for values, dtype in zip(outputs, dtypes, strict=True):
        values = np.array(values, dtype=np.float64)
        values[:lookback] = 0.0 if dtype == "int32" else np.nan
        blanked.append(values)
    return blanked


def cell(value: float, dtype: str) -> str:
    return str(int(value)) if dtype == "int32" else repr(float(value))


def evaluated(functions: str) -> str:
    """Name the TA-Lib functions that produced the numbers, or say that none did."""
    import talib

    if functions.startswith("talib."):
        return f"evaluated through ta-lib-python {talib.__version__} {functions}"
    return (
        f"evaluated as {functions}; no ta-lib-python {talib.__version__} function computes a step"
    )


def write_case(name: str, case: str, spec: dict, params: dict) -> Path:

    dataset = dataset_for(name, case)
    columns = read_columns(dataset)
    outputs, lookback, functions = FORMULAS[name](columns, params)
    if len(outputs) != len(spec["outputs"]):
        raise SystemExit(
            f"{name}: formula returned {len(outputs)} outputs, spec has {len(spec['outputs'])}"
        )
    dtypes = [o["dtype"] for o in spec["outputs"]]
    outputs = warm_up(outputs, lookback, dtypes)

    rendered = ", ".join(f"{key}={value}" for key, value in params.items()) or "none"
    header = [
        f"# indicator: {name}",
        f"# case: {case}",
        f"# params: {rendered}",
        f"# oracle: oracle A, the formula of INDICATORS.md section 5.3 {evaluated(functions)}",
        f"# produced_by: python scripts/oracle/formula_golden.py {name} --case {case}",
        f"# input: testdata/{dataset} (all rows)",
        f"# tolerance: {TOLERANCE}",
        "# excluded_rows: none",
        f"# note: rows before {lookback} are warm-up for every output (CONVENTIONS.md section 2)",
        f"# date: {dt.date.today().isoformat()}",
    ]
    names = [i["name"] for i in spec["inputs"]] + [o["name"] for o in spec["outputs"]]
    lines = [*header, ",".join(names)]
    for row in range(len(columns["close"])):
        cells = [repr(float(columns[i["name"]][row])) for i in spec["inputs"]]
        cells += [cell(values[row], dtype) for values, dtype in zip(outputs, dtypes, strict=True)]
        lines.append(",".join(cells))

    path = INDICATORS / name / "golden" / f"{case}.csv"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
    size = path.stat().st_size
    if size > 2_000_000:
        raise SystemExit(
            f"{path.name} is {size} bytes; docs/SPEC_FORMAT.md section 4 caps it at 2 MB"
        )
    return path


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("indicator", help="folder name under crates/trendlib/src/indicators")
    parser.add_argument("--case", help="one case; default is every case")
    args = parser.parse_args()

    try:
        import talib  # noqa: F401
        import yaml  # noqa: F401
    except ImportError as exc:
        raise SystemExit(
            f"{exc.name} is missing; install the dev extra: pip install -e '.[dev]'"
        ) from exc

    if args.indicator not in FORMULAS:
        raise SystemExit(
            f"{args.indicator} has no formula here; see docs/INDICATORS.md section 5.3"
        )
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
