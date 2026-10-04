"""The factory behind `tl.stream.<name>`.

Hand-written; `stream.py` is generated and holds one Factory per indicator.
"""

from __future__ import annotations

from typing import Any

from trendlib import _convert


class Factory:
    """`tl.stream.<name>(history, **params)` plus `.open_and_fill(...)`."""

    def __init__(
        self,
        name: str,
        handle: type,
        inputs: tuple[str, ...],
        kinds: tuple[str, ...],
        outputs: tuple[str, ...],
    ) -> None:
        self._name = name
        self._handle = handle
        self._inputs = inputs
        self._kinds = kinds
        self._outputs = outputs
        self.__name__ = name
        self.__qualname__ = f"stream.{name}"
        self.__doc__ = (
            f"Open a {name} stream on history.\n\n"
            f"Inputs: {', '.join(inputs)}. A DataFrame holding them all may be\n"
            f"given instead. History needs at least\n"
            f"`tl.lookback({name!r}, **params) + 1` valid bars; fewer raises\n"
            f"InsufficientHistory. Parameters are fixed for the stream's life."
        )

    def _resolve(self, args: tuple[Any, ...], params: dict[str, Any]):
        padded = args + (None,) * (len(self._inputs) - len(args))
        if len(padded) != len(self._inputs):
            raise TypeError(
                f"stream.{self._name} takes {len(self._inputs)} inputs "
                f"({', '.join(self._inputs)}), got {len(args)}"
            )
        columns, carrier = _convert.bars(self._name, padded, self._inputs, self._kinds)
        return columns, carrier, self._check(params)

    def _check(self, params: dict[str, Any]) -> dict[str, Any]:
        from trendlib import _core

        known = _core.PARAMS.get(self._name, {})
        unknown = set(params) - set(known)
        if unknown:
            raise TypeError(f"stream.{self._name}: unexpected parameter {sorted(unknown)[0]!r}")
        resolved = {name: spec["default"] for name, spec in known.items()}
        resolved.update(params)
        return resolved

    def __call__(self, *args: Any, **params: Any) -> Any:
        columns, _, resolved = self._resolve(args, params)
        return self._handle.open(*columns, **resolved)

    def open_and_fill(self, *args: Any, **params: Any) -> tuple[Any, Any]:
        """Return `(stream, batch_outputs)` in one pass over the history."""
        columns, carrier, resolved = self._resolve(args, params)
        handle, out = self._handle.open_and_fill(*columns, **resolved)
        return handle, _convert.wrap_outputs(out, carrier, self._outputs)

    def __repr__(self) -> str:
        return f"<trendlib stream factory {self._name}>"
