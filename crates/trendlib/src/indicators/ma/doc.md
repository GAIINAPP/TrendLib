# Moving Average

The average of the last `period` values of a series, smoothed by whichever
moving average `ma_type` names. It is the dispatcher the band and oscillator
indicators are built on, so one call covers every average the library has
rather than one function per average.

## Formula

$$
\mathrm{ma}_t = \mathrm{MA}_{\text{type}}(x, n)_t
$$

where $x$ is `source`, $n$ is `period` and $\mathrm{MA}_{\text{type}}$ is the
average `ma_type` selects.

## Conventions

- Warm-up is the chosen average's own warm-up: `n - 1` bars for `sma`, `ema`,
  `wma`, `trima` and `rma`, `2(n - 1)` for `dema` and `3(n - 1)` for `tema`.
- With `period=1` every average returns the input unchanged and the lookback is
  zero.
- `ema`, `rma`, `dema` and `tema` are recursive, so a result computed from a
  slice of the data differs from the same rows of a full run
  (`CONVENTIONS.md` § 5). `sma`, `wma` and `trima` are not.
- An average that the library does not implement yet is refused by name rather
  than replaced by another one (`INDICATORS.md` § 1).

## Example

```python
import trendlib as tl
smoothed = tl.ma(close, period=30, ma_type="ema")
```

## References

- TA-Lib, `ta_MA.c`, moving average dispatch over the MA type table.
