# Inside Day

Marks a bar whose range lies inside the previous bar's: a lower high and a
higher low. It has no direction, so it reads +100 wherever the shape is.

## Formula

$$
\text{bar\_inside\_day}_t = 100\,[H_t < H_{t-1} \wedge L_t > L_{t-1}]
$$

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `inside_day`
  (oracle P, `INDICATORS.md` § 5.2).
- The shape has no direction; `+100` marks it, as the oracle's `+1` does.
- Warm-up is 1 bar, the first bar the oracle reads. An `int32` column warms up
  with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what price did
  on these bars; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.bar_inside_day(open, high, low, close)
```

## References

- Laurence A. Connors and Linda Bradford Raschke, Street Smarts, M. Gordon Publishing, 1995
- ta-patterns 1.2.1, ta_patterns.chart_patterns.inside_day (oracle P)
