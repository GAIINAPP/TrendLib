# Percentage Price Oscillator

The gap between a short and a long moving average, divided by the long one and
read as a percentage. Dividing makes the value comparable across instruments and
across time in a way the absolute gap is not.

## Formula

$$
\mathrm{ppo}_t = 100 \cdot
\frac{\mathrm{MA}_{\text{type}}(x, n_{fast})_t - \mathrm{MA}_{\text{type}}(x, n_{slow})_t}
     {\mathrm{MA}_{\text{type}}(x, n_{slow})_t}
$$

where $x$ is `source`, $n_{fast}$ is `fast_period` and $n_{slow}$ is
`slow_period`.

## Conventions

- The two periods are sorted before anything is computed, so `fast_period=26,
  slow_period=12` returns the same values as `fast_period=12, slow_period=26`.
- A slow average of exactly zero gives `0.0`, as TA-Lib does. The test is on the
  exact value: an average that is merely small is a genuinely large percentage
  and is reported as one.
- Warm-up is the longer period's own warm-up for the chosen average.
- The default average is `ema`, as TA-Lib's.

## Example

```python
import trendlib as tl
spread = tl.ppo(close, fast_period=12, slow_period=26, ma_type="ema")
```

## References

- Gerald Appel, *Technical Analysis - Power Tools for Active Investors*, FT Press, 2005.
