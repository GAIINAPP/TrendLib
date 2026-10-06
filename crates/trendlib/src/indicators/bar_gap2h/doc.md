# Gap Beyond Two Bars

Marks a bar that opens beyond both of the two bars before it: above both of
their highs (+100) or below both of their lows (-100).

## Formula

$$
\text{bar\_gap2h}_t = 100\,[O_t > \max(H_{t-1}, H_{t-2})] - 100\,[O_t < \min(L_{t-1}, L_{t-2})]
$$

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `gap2h_combined`
  (oracle P, `INDICATORS.md` § 5.2).
- The two directions are the oracle's two halves combined its own way: each adds
  its sign and the sum is clipped, so a bar that answers both reads `0`.
- Warm-up is 2 bars, the first bar the oracle reads. An `int32` column warms up
  with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what price did
  on these bars; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.bar_gap2h(open, high, low, close)
```

## References

- Laurence A. Connors and Linda Bradford Raschke, Street Smarts, M. Gordon Publishing, 1995
- ta-patterns 1.2.1, ta_patterns.chart_patterns.gap2h_combined (oracle P)
