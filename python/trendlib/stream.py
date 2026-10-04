"""Live indicator values.

A stream is a value, not a session. It holds no global state, a `copy` is an
independent fork, and the values it returns are bitwise identical to running
the batch function over the whole extended series
(`docs/PYTHON_API.md` section 4).

Hand-written for M1; generated from the specs in M2.
"""

from __future__ import annotations

from typing import Any

from trendlib import _convert, _core

__all__ = ["ema", "rsi", "sma"]


class _Factory:
    """`tl.stream.<name>(history, **params)` plus `.open_and_fill(...)`."""

    def __init__(self, name: str, handle: type) -> None:
        self._name = name
        self._handle = handle
        self.__name__ = name
        self.__qualname__ = f"stream.{name}"
        self.__doc__ = (
            f"Open a {name} stream on `history`.\n\n"
            f"`history` is an array-like, Series or DataFrame with at least\n"
            f"`tl.lookback({name!r}, **params) + 1` valid bars; fewer raises\n"
            f"InsufficientHistory. Parameters are fixed for the stream's life."
        )

    def _prepare(self, history: Any, params: dict[str, Any]) -> tuple[Any, dict[str, Any]]:
        values, _ = _convert.as_series(self._name, history)
        period = params.pop("period", _core.PARAMS[self._name]["period"]["default"])
        if params:
            raise TypeError(f"stream.{self._name}: unexpected parameter {sorted(params)[0]!r}")
        return values, {"period": _convert.as_int(self._name, "period", period)}

    def __call__(self, history: Any, **params: Any) -> Any:
        values, kwargs = self._prepare(history, dict(params))
        return self._handle.open(values, **kwargs)

    def open_and_fill(self, history: Any, **params: Any) -> tuple[Any, Any]:
        """Return `(stream, batch_outputs)` in one pass over `history`."""
        carrier = _convert.carrier_of(self._name, history)
        values, kwargs = self._prepare(history, dict(params))
        handle, out = self._handle.open_and_fill(values, **kwargs)
        return handle, _convert.wrap(out, carrier, self._name)


sma = _Factory("sma", _core.SmaStream)
ema = _Factory("ema", _core.EmaStream)
rsi = _Factory("rsi", _core.RsiStream)
