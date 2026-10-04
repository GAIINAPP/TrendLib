# Moving Average with Variable Period

A moving average whose length each bar chooses for itself, from a second input
holding the period to use.

## Formula

$$
\mathrm{mavp}_t = \mathrm{MA}_{\text{type}}(x, n_t)_t
\qquad
n_t = \left\lfloor \mathrm{clamp}(p_t,\ n_{\min},\ n_{\max}) \right\rfloor
$$

where $p$ is `periods`, and the fraction is dropped rather than rounded.

## Conventions

- Warm-up is the longest average's own, since any bar may ask for it.
- A period is clamped to `[min_period, max_period]` and then truncated, and a
  clamped period of zero is read as one.
- Each distinct period is a separate average, started far enough back to have
  its own warm-up behind this indicator's first row and no further. That is
  what TA-Lib does, and it is what a recursive average's seed depends on: an
  `ema` asked for at bar 500 is not the one that would have run from bar 0.
- Setting `min_period` above `max_period` is allowed by their ranges and asks
  for an average longer than the warm-up covers. The longest is used instead,
  where TA-Lib reads past the start of its own buffer.
- This is the one indicator whose stream keeps the whole series. A period asked
  for the first time at bar five thousand still has to be the average over the
  bars before it, so those bars have to still be there.
- Not path dependent with `sma`, `wma` or `trima`; path dependent with the
  recursive averages, as they are.

## Example

```python
import trendlib as tl
average = tl.mavp(close, periods, min_period=2, max_period=30)
```

## References

- TA-Lib, `ta_MAVP.c`, one moving average per distinct period requested.
