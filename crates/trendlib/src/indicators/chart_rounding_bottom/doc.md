# Rounding Bottom

Marks a close above a saucer: over the last `period` closes the middle third
averaged at least `min_depth` below both outer thirds, and the close is now
above the higher of the two.

## Formula

Over the closes of the `period` bars before $t$, split into a first
third (`period / 3` closes), a middle third (the next `period / 3`) and the rest,
let $A$, $M$ and $Z$ be their means.

$$
\text{chart\_rounding\_bottom}_t = 100\,\Big[M < \min(A, Z) \wedge \frac{\min(A, Z) - M}{A} \ge d
\wedge C_t > \max(A, Z)\Big]
$$

where $d$ is `min_depth`. The depth is measured against the first third's mean,
as the oracle measures it.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `rounding_bottom`
  with `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
- The reading is repeated on every bar the conditions hold, not only the first.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did relative to the shape; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.chart_rounding_bottom(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.rounding_bottom (oracle P)
