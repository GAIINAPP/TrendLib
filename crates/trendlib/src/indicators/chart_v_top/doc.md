# V-Top

Marks an inverted V: a rise of at least `min_rise` through the first half of the
last `period` closes, a fall of at least four fifths of that through the second,
and a close back within 2 percent of where it began.

## Formula

Over the closes of the $p$ bars before $t$, with $h = \lfloor p/2 \rfloor$, the
first $p - h$ closes the left arm, the last $h$ the right, $S$ the first close
and $E$ the last:

$$
\text{chart\_v\_top}_t = -100\,\Big[\frac{\max_{\text{left}} - S}{S} \ge r
\wedge \frac{\max_{\text{right}} - E}{S} \ge 0.8\,r \wedge C_t \le 1.02\,S\Big]
$$

where $p$ is `period` and $r$ is `min_rise`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `v_top` with
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
found = tl.chart_v_top(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.v_top (oracle P)
