# Absolute Price Oscillator

The gap between a short and a long moving average of the same series, in the
units of the series itself. It is MACD's line without the signal and histogram,
and with the choice of average left open.

## Formula

$$
\mathrm{apo}_t = \mathrm{MA}_{\text{type}}(x, n_{fast})_t - \mathrm{MA}_{\text{type}}(x, n_{slow})_t
$$

where $x$ is `source`, $n_{fast}$ is `fast_period` and $n_{slow}$ is
`slow_period`.

## Conventions

- The two periods are sorted before anything is computed, so `fast_period=26,
  slow_period=12` returns the same values as `fast_period=12, slow_period=26`
  rather than their negation. TA-Lib does the same.
- Warm-up is the longer period's own warm-up for the chosen average, so both
  averages have a value on the first row that has one.
- The default average is `ema`, as TA-Lib's.

## Example

```python
import trendlib as tl
spread = tl.apo(close, fast_period=12, slow_period=26, ma_type="ema")
```

## References

- Gerald Appel, *Technical Analysis - Power Tools for Active Investors*, FT Press, 2005.
