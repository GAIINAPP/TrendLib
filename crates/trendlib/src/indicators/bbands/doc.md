# Bollinger Bands

A moving average with a band drawn a number of standard deviations above and
below it. The bands widen as the series becomes more variable and narrow as it
settles, so the same distance from the average means something different in a
quiet market than in a busy one.

## Formula

$$
\text{middle}_t = \mathrm{MA}_{\text{type}}(x, n)_t \qquad
\sigma_t = \sqrt{\tfrac{1}{n}\textstyle\sum_{i=t-n+1}^{t} (x_i - \mathrm{SMA}(x,n)_t)^2}
$$

$$
\text{upper}_t = \text{middle}_t + k_{up}\,\sigma_t \qquad
\text{lower}_t = \text{middle}_t - k_{dn}\,\sigma_t
$$

where $x$ is `source`, $n$ is `period`, $k_{up}$ is `nbdev_up` and $k_{dn}$ is
`nbdev_dn`.

## Conventions

- Population standard deviation, dividing by $n$, as TA-Lib.
- The deviation is always measured about the window's **simple** mean, even when
  `ma_type` makes the middle band something else. With `ma_type="ema"` the band
  is therefore not centred on the mean the width was computed from. This is
  TA-Lib's behaviour and is kept for parity.
- Warm-up is the chosen average's: `n - 1` bars for `sma`, `ema`, `wma`, `trima`
  and `rma`, `2(n - 1)` for `dema`, `3(n - 1)` for `tema`. The deviation is
  ready at `n - 1` in every case, so it never decides the first row.
- `nbdev_up` and `nbdev_dn` are independent; a negative one puts the band on the
  other side of the average, and the outputs are not reordered.
- With `ma_type="ema"`, `"rma"`, `"dema"` or `"tema"` the result is path
  dependent (`CONVENTIONS.md` § 5); with the others it is not.
- Squaring a distance overflows above about `1e154`, so a finite series of that
  magnitude gives infinite or `NaN` bands. TA-Lib's STDDEV overflows at the same
  point and returns `NaN` too.

## Example

```python
import trendlib as tl
upper, middle, lower = tl.bbands(close, period=20)
```

## References

- John Bollinger, *Bollinger on Bollinger Bands*, McGraw-Hill, 2001.
