"""Conversion between the caller's containers and the plain arrays `_core` takes.

Everything the extension module sees is a C-contiguous float64 NumPy array, so
all pandas and polars knowledge lives here and the Rust side stays small
(`docs/ARCHITECTURE.md`, Python layer).

Optional libraries are looked up in `sys.modules` instead of being imported. A
caller holding a DataFrame has already imported the library that made it, so
this recognises their objects without paying for an import they did not ask for
and without a module-level import cache.
"""

from __future__ import annotations

import sys
from typing import Any

import numpy as np

from trendlib.errors import InvalidInput

__all__ = ["as_series", "carrier_of", "wrap"]

OHLCV = ("open", "high", "low", "close", "volume")
TIMESTAMP_COLUMNS = ("timestamp", "datetime", "date", "time")


def _pandas():
    return sys.modules.get("pandas")


def _polars():
    return sys.modules.get("polars")


def _is_pandas(obj: Any, attribute: str) -> bool:
    pd = _pandas()
    return pd is not None and isinstance(obj, getattr(pd, attribute))


def _is_polars(obj: Any, attribute: str) -> bool:
    pl = _polars()
    return pl is not None and isinstance(obj, getattr(pl, attribute))


class Carrier:
    """How to give a result back in the shape the caller handed us."""

    __slots__ = ("index", "kind")

    def __init__(self, kind: str, index: Any = None) -> None:
        self.kind = kind
        self.index = index


def _frame_column(indicator: str, frame: Any, wanted: str, columns: list[Any]) -> Any:
    lookup = {str(name).lower(): name for name in columns}
    if wanted not in lookup:
        raise InvalidInput(
            f"{indicator}: the frame has no {wanted!r} column; "
            f"columns are {[str(c) for c in columns]}"
        )
    return frame[lookup[wanted]]


def as_series(indicator: str, source: Any, column: str = "close") -> tuple[np.ndarray, Carrier]:
    """Return `(values, carrier)` for a single-series input.

    A DataFrame contributes its `close` column, matched case-insensitively
    (`docs/PYTHON_API.md` section 1).
    """
    if _is_pandas(source, "DataFrame"):
        frame = source
        series = _frame_column(indicator, frame, column, list(frame.columns))
        return _to_float64(indicator, series.to_numpy()), Carrier("pandas", frame.index)

    if _is_pandas(source, "Series"):
        return _to_float64(indicator, source.to_numpy()), Carrier("pandas", source.index)

    if _is_polars(source, "DataFrame"):
        frame = source
        series = _frame_column(indicator, frame, column, list(frame.columns))
        return _to_float64(indicator, series.to_numpy()), Carrier("polars")

    if _is_polars(source, "Series"):
        return _to_float64(indicator, source.to_numpy()), Carrier("polars")

    return _to_float64(indicator, source), Carrier("numpy")


def carrier_of(indicator: str, source: Any) -> Carrier:
    return as_series(indicator, source)[1]


def _to_float64(indicator: str, values: Any) -> np.ndarray:
    try:
        array = np.ascontiguousarray(values, dtype=np.float64)
    except (TypeError, ValueError) as exc:
        raise InvalidInput(f"{indicator}: input is not a numeric array ({exc})") from exc

    if array.ndim == 0:
        raise InvalidInput(f"{indicator}: input is a scalar, not a series")
    if array.ndim != 1:
        raise InvalidInput(
            f"{indicator}: input must be one-dimensional, got {array.ndim} dimensions"
        )
    return array


def wrap(values: np.ndarray, carrier: Carrier, name: str) -> Any:
    """Give a single output back in the caller's container type."""
    if carrier.kind == "pandas":
        pd = _pandas()
        return pd.Series(values, index=carrier.index, name=name)
    if carrier.kind == "polars":
        pl = _polars()
        return pl.Series(name, values)
    return values


def as_int(indicator: str, param: str, value: Any) -> int:
    """Accept a Python or NumPy integer and nothing else.

    Range checking stays in Rust, which owns the documented bounds and writes
    the message `docs/PYTHON_API.md` specifies.
    """
    if isinstance(value, bool) or not isinstance(value, (int, np.integer)):
        kind = type(value).__name__
        raise InvalidInput(f"{indicator}: {param} must be an integer, got {kind}")
    return int(value)
