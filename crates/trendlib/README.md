# trendlib

325 technical-analysis indicators and patterns, with no runtime dependencies.

This crate is the computation core of
[TrendLib](https://github.com/GAIINAPP/TrendLib). It forbids `unsafe`, pulls in
nothing at runtime, and holds one kernel per indicator: a step that takes a bar
and returns the next value. The batch function folds that step over a slice and
the streaming form calls it per bar, so the two agree bitwise by construction.

The Python package `trendlib` wraps this crate through PyO3 and is where most
users will meet it.

Covered: overlap studies, momentum, volatility, volume, price transforms, the
Hilbert-transform cycle family, statistics, math transforms and operators, 61
candlestick patterns, 63 chart patterns, 19 bar patterns, 12 harmonic patterns
and 7 pivot-level families.

## License

Apache License 2.0. See the LICENSE file at the repository root.
