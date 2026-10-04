"""Batch functions and their TA-Lib aliases.

Hand-written for M1; `cargo xtask generate` writes this file from every
`spec.yaml` in M2. Defaults and ranges are read from the extension module
rather than repeated here, so there is one place they can be wrong.
"""

from __future__ import annotations

from typing import Any

from trendlib import _convert, _core

__all__ = ["EMA", "RSI", "SMA", "ema", "lookback", "rsi", "sma"]

_PARAMS = _core.PARAMS


def _period_default(name: str) -> int:
    return _PARAMS[name]["period"]["default"]


def _single_series(name: str, source: Any, period: Any) -> Any:
    values, carrier = _convert.as_series(name, source)
    period = _convert.as_int(name, "period", period)
    out = getattr(_core, name)(values, period=period)
    return _convert.wrap(out, carrier, name)


def sma(source: Any, *, period: int = _period_default("sma")) -> Any:
    """Simple moving average.

    The unweighted mean of the last `period` values. It smooths a series by
    giving every bar in the window the same weight.

    Parameters
    ----------
    source : array-like, Series or DataFrame
        Values to average. A DataFrame contributes its ``close`` column.
    period : int, default 30
        Number of bars in the average, from 1 to 100000.

    Returns
    -------
    ndarray, Series
        The same length as `source`, with `period - 1` warm-up rows of NaN.
        A pandas input keeps its index and the output is named ``sma``.
    """
    return _single_series("sma", source, period)


def ema(source: Any, *, period: int = _period_default("ema")) -> Any:
    """Exponential moving average.

    A weighted mean in which each older bar counts less than the one after it,
    by a constant factor. Seeded with the simple mean of the first `period`
    values, as TA-Lib seeds it.

    Parameters
    ----------
    source : array-like, Series or DataFrame
        Values to average. A DataFrame contributes its ``close`` column.
    period : int, default 30
        Number of bars the smoothing factor is derived from, 1 to 100000.

    Returns
    -------
    ndarray, Series
        The same length as `source`, with `period - 1` warm-up rows of NaN.
    """
    return _single_series("ema", source, period)


def rsi(source: Any, *, period: int = _period_default("rsi")) -> Any:
    """Relative strength index.

    The share of recent movement that was upward, on a scale from 0 to 100.
    Gains and losses are averaged over `period` bar-to-bar changes with
    Wilder's smoothing.

    Parameters
    ----------
    source : array-like, Series or DataFrame
        Values to measure. A DataFrame contributes its ``close`` column.
    period : int, default 14
        Number of changes the averages are taken over, from 2 to 100000.

    Returns
    -------
    ndarray, Series
        The same length as `source`, with `period` warm-up rows of NaN.
    """
    return _single_series("rsi", source, period)


def lookback(name: str, **params: Any) -> int:
    """Warm-up rows before the first defined value.

    >>> import trendlib as tl
    >>> tl.lookback("rsi", period=14)
    14
    """
    unknown = set(params) - {"period"}
    if unknown:
        raise TypeError(f"lookback: {name} has no parameter {sorted(unknown)[0]!r}")
    period = params.get("period")
    if period is not None:
        period = _convert.as_int(name, "period", period)
    return _core.lookback(name, period=period)


# TA-Lib aliases (D12): ta-lib-python's parameter names, identical computation.


def SMA(real: Any, timeperiod: int = _period_default("sma")) -> Any:  # noqa: N802
    """TA-Lib-style alias for :func:`sma`."""
    return sma(real, period=timeperiod)


def EMA(real: Any, timeperiod: int = _period_default("ema")) -> Any:  # noqa: N802
    """TA-Lib-style alias for :func:`ema`."""
    return ema(real, period=timeperiod)


def RSI(real: Any, timeperiod: int = _period_default("rsi")) -> Any:  # noqa: N802
    """TA-Lib-style alias for :func:`rsi`."""
    return rsi(real, period=timeperiod)
