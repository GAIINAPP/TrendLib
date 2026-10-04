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

__all__ = ["as_series", "bars", "carrier_of", "wrap", "wrap_outputs"]

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


def _frame_of(indicator: str, value: Any) -> tuple[Any, list[Any], str] | None:
    """`(frame, columns, kind)` when `value` is a DataFrame, else None."""
    if _is_pandas(value, "DataFrame"):
        return value, list(value.columns), "pandas"
    if _is_polars(value, "DataFrame"):
        return value, list(value.columns), "polars"
    return None


def bars(
    indicator: str,
    values: tuple[Any, ...],
    names: tuple[str, ...],
    kinds: tuple[str, ...],
) -> tuple[list[np.ndarray], Carrier]:
    """Resolve an indicator's inputs, however the caller supplied them.

    A single DataFrame may stand in for every bar input, with columns matched
    case-insensitively (`docs/PYTHON_API.md` section 1). Otherwise each input is
    given separately and they must all be present and the same length.
    """
    frame = _frame_of(indicator, values[0]) if values else None
    if frame is not None and all(value is None for value in values[1:]):
        # A frame can say which column is the high and which is the close, but
        # it cannot say which of two interchangeable series is which. Mapping
        # both to `close` would make `sub` return zeros on every row, which is
        # the kind of plausible wrong number a loud error is worth avoiding.
        if sum(1 for kind in kinds if kind == "series") > 1:
            raise InvalidInput(
                f"{indicator}: a DataFrame cannot say which column is "
                f"{names[0]} and which is {names[1]}; pass them separately"
            )
        held, columns, kind = frame
        index = held.index if kind == "pandas" else None
        resolved = []
        for name, wanted in zip(names, kinds, strict=True):
            column = "close" if wanted == "series" else wanted
            series = _frame_column(indicator, held, column, columns)
            resolved.append(_to_float64(f"{indicator}: {name}", series.to_numpy()))
        _equal_lengths(indicator, names, resolved)
        return resolved, Carrier(kind, index)

    missing = [name for name, value in zip(names, values, strict=True) if value is None]
    if missing:
        raise InvalidInput(
            f"{indicator}: missing input {missing[0]!r}; "
            f"give every input, or one DataFrame holding {', '.join(names)}"
        )

    carrier = as_series(indicator, values[0], "close" if kinds[0] == "series" else kinds[0])[1]
    resolved = [
        as_series(f"{indicator}: {name}", value, "close" if kind == "series" else kind)[0]
        for name, kind, value in zip(names, kinds, values, strict=True)
    ]
    _equal_lengths(indicator, names, resolved)
    return resolved, carrier


def _equal_lengths(indicator: str, names: tuple[str, ...], columns: list[np.ndarray]) -> None:
    first = len(columns[0])
    for name, column in zip(names, columns, strict=True):
        if len(column) != first:
            raise InvalidInput(
                f"{indicator}: inputs must have equal length; {names[0]} has {first} rows, "
                f"{name} has {len(column)}"
            )


def wrap_outputs(values: Any, carrier: Carrier, names: tuple[str, ...]) -> Any:
    """Give one or several outputs back in the caller's container type."""
    if len(names) == 1:
        return wrap(values, carrier, names[0])

    columns = dict(zip(names, values, strict=True))
    if carrier.kind == "pandas":
        pd = _pandas()
        return pd.DataFrame(columns, index=carrier.index)
    if carrier.kind == "polars":
        pl = _polars()
        return pl.DataFrame(columns)
    return tuple(values)


def as_float(indicator: str, param: str, value: Any) -> float:
    """Accept anything that is a real number and nothing else.

    Range checking stays in Rust, which owns the documented bounds and writes
    the message `docs/PYTHON_API.md` specifies.
    """
    if isinstance(value, bool) or not isinstance(value, (int, float, np.integer, np.floating)):
        kind = type(value).__name__
        raise InvalidInput(f"{indicator}: {param} must be a number, got {kind}")
    return float(value)
