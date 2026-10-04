# Acceleration Bands

Bands drawn by widening each bar's high and low in proportion to how wide the
bar itself was, then averaging. A run of wide bars pushes the bands out faster
than a band built from a deviation does.

## Formula

$$
r_t = \frac{4\,(H_t - L_t)}{H_t + L_t}
$$

$$
\text{upper}_t = \mathrm{SMA}\big(H (1 + r),\, n\big)_t \qquad
\text{middle}_t = \mathrm{SMA}(C, n)_t \qquad
\text{lower}_t = \mathrm{SMA}\big(L (1 - r),\, n\big)_t
$$

where $n$ is `period`.

## Conventions

- Warm-up is `period - 1` bars.
- A bar whose high and low sum to zero has no midpoint for the ratio to be
  measured against, and the row is non-finite rather than guarded. Prices do
  not reach there; a series centred on zero can.
- The widening is per bar, not per window: each bar is widened by its own
  range before the average is taken.
- Not path dependent.

## Example

```python
import trendlib as tl
upper, middle, lower = tl.accbands(high, low, close, period=20)
```

## References

- Price Headley, *Big Trends in Trading*, Wiley, 2002.
