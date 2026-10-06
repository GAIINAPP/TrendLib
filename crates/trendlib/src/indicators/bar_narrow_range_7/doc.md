# Narrow Range 7

Marks a bar whose high-low range is the narrowest of the last 7 bars, this one
included. It has no direction, so it reads +100 wherever the shape is.

## Formula

$$
\text{bar\_narrow\_range\_7}_t = 100\,\Big[H_t - L_t = \min_{t-6 \le j \le t} (H_j - L_j)\Big]
$$

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `narrow_range_7`
  (oracle P, `INDICATORS.md` § 5.2).
- A tie for narrowest counts, as it does in the oracle.
- The shape has no direction; `+100` marks it, as the oracle's `+1` does.
- Warm-up is 6 bars, the first bar the oracle reads. An `int32` column warms up
  with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what price did
  on these bars; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.bar_narrow_range_7(open, high, low, close)
```

## References

- Toby Crabel, Day Trading with Short Term Price Patterns and Opening Range Breakout, Traders Press, 1990
- ta-patterns 1.2.1, ta_patterns.chart_patterns.narrow_range_7 (oracle P)
