# Rounding Top

Marks a close below a dome: over the last `period` closes the middle third
averaged at least `min_depth` above both outer thirds, and the close is now
below the lower of the two.

## Formula

Over the closes of the `period` bars before $t$, split into a first
third (`period / 3` closes), a middle third (the next `period / 3`) and the rest,
let $A$, $M$ and $Z$ be their means.

$$
\text{chart\_rounding\_top}_t = -100\,\Big[M > \max(A, Z) \wedge \frac{M - \max(A, Z)}{M} \ge d
\wedge C_t < \min(A, Z)\Big]
$$

where $d$ is `min_depth`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `rounding_top`
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
found = tl.chart_rounding_top(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.rounding_top (oracle P)
