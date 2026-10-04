"""Exceptions raised by TrendLib.

The hierarchy is part of the public API (``docs/PYTHON_API.md`` section 1):
every error is a ``TrendLibError``, and the two that report bad arguments are
also ``ValueError`` so existing ``except ValueError`` handlers keep working.
"""

__all__ = ["InsufficientHistory", "InvalidInput", "TrendLibError"]


class TrendLibError(Exception):
    """Base class for every error TrendLib raises."""


class InvalidInput(TrendLibError, ValueError):
    """A parameter is out of range, or an input violates the frame contract.

    Raised for an out-of-range parameter, an unknown enum value, inputs of
    unequal length, a NaN or infinity after the first valid bar, a negative
    volume, or timestamps that are naive or not non-decreasing.
    """


class InsufficientHistory(TrendLibError, ValueError):
    """A stream was opened with fewer bars than its lookback requires.

    The message states how many valid bars the parameters need.
    """
