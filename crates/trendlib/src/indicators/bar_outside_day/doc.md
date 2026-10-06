# Outside Day

Marks a bar whose range covers the previous bar's on both sides and that closes
beyond it: above the previous high (+100) or below the previous low (-100).

## Formula

With $X_t = H_t > H_{t-1} \wedge L_t < L_{t-1}$:

$$
\text{bar\_outside\_day}_t = 100\,[X_t \wedge C_t > H_{t-1}] - 100\,[X_t \wedge C_t < L_{t-1}]
$$

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `outside_day`
  (oracle P, `INDICATORS.md` § 5.2).
- The two directions are the oracle's two halves combined its own way: each adds
  its sign and the sum is clipped, so a bar that answers both reads `0`.
- Warm-up is 1 bar, the first bar the oracle reads. An `int32` column warms up
  with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what price did
  on these bars; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.bar_outside_day(open, high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.outside_day (oracle P)
