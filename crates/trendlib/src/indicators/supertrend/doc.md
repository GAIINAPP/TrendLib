# Supertrend

A trailing line that follows the lower band while the trend is up and the upper one while it is down.

## Formula

With $\mathrm{mid}_t = (H_t + L_t)/2$ and $k$ the `multiplier`,

$$
\text{upper}_t = \mathrm{mid}_t + k\,\mathrm{ATR}_t \qquad
\text{lower}_t = \mathrm{mid}_t - k\,\mathrm{ATR}_t
$$

Each band is carried forward unless the new one is closer to price or the
previous close has already passed it. The trend turns down when the close
falls below the carried lower band and up when it rises above the carried
upper one.

## Conventions

- Warm-up is `period` bars, the average true range's own.
- The first row has no trend to carry and starts rising, which is TA-Lib's
  seed. TradingView seeds from the first close instead and signs its direction
  the other way round (`CONVENTIONS.md` § 10).
- `supertrend_direction` is `+1` while the lower band is followed and `-1`
  while the upper is; an `int32` column warms up with `0`, not `NaN`.
- Recursive on both the range average and the carried bands, so the result is
  path dependent (`CONVENTIONS.md` § 5).

## Example

```python
import trendlib as tl
supertrend, supertrend_direction = tl.supertrend(high, low, close)
```

## References

- Olivier Seban, as described in INDICATORS.md section 3.5
