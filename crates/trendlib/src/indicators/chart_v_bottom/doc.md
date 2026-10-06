# V-Bottom

Marks a V: a fall of at least `min_drop` through the first half of the last
`period` closes, a recovery of at least four fifths of that through the second,
and a close back within 2 percent of where it began.

## Formula

Over the closes of the $p$ bars before $t$, with $h = \lfloor p/2 \rfloor$, the
first $p - h$ closes the left arm, the last $h$ the right, $S$ the first close
and $E$ the last:

$$
\text{chart\_v\_bottom}_t = 100\,\Big[\frac{S - \min_{\text{left}}}{S} \ge d
\wedge \frac{E - \min_{\text{right}}}{S} \ge 0.8\,d \wedge C_t \ge 0.98\,S\Big]
$$

where $p$ is `period` and $d$ is `min_drop`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `v_bottom` with
  `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
- The four fifths and the 2 percent are the oracle's constants, not parameters.
- The reading is repeated on every bar the conditions hold, not only the first.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did relative to the shape; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.chart_v_bottom(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.v_bottom (oracle P)
