"""TrendLib: fast, tested technical-analysis indicators with a Rust core.

TrendLib computes measurements from price and volume series. It does not give
investment advice.
"""

import re as _re

from trendlib import _core, stream
from trendlib._functions import EMA, RSI, SMA, ema, lookback, rsi, sma
from trendlib.errors import InsufficientHistory, InvalidInput, TrendLibError

__all__ = [
    "EMA",
    "RSI",
    "SMA",
    "InsufficientHistory",
    "InvalidInput",
    "TrendLibError",
    "__version__",
    "__version_info__",
    "ema",
    "lookback",
    "rsi",
    "sma",
    "stream",
]


def _version_info(version: str) -> tuple[int, ...]:
    # maturin maps a Cargo pre-release (0.1.0-alpha.1) to a PEP 440 version
    # (0.1.0a1); __version_info__ carries the release part of it only.
    release = _re.match(r"\d+(?:\.\d+)*", version)
    if release is None:
        return ()
    return tuple(int(part) for part in release.group(0).split("."))


__version__: str = _core.__version__
__version_info__: tuple[int, ...] = _version_info(__version__)
