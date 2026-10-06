# Horn Bottom

Marks a horn bottom: two bars two apart whose lows match within `tol`, the bar
between them at least 1 percent higher, and a close above the middle bar's
close.

## Formula

$$
\text{bar\_horn\_bottom}_t = 100\,\Big[\frac{|L_t - L_{t-2}|}{\max(|L_t|, |L_{t-2}|)} < \tau
\wedge L_{t-1} > 1.01 \max(L_t, L_{t-2}) \wedge C_t > C_{t-1}\Big]
$$

where $\tau$ is `tol`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `horn_bottom`
  (oracle P, `INDICATORS.md` § 5.2).
- The 1 percent margin of the middle bar is the oracle's constant, not a
  parameter.
- Warm-up is 2 bars, the first bar the oracle reads. An `int32` column warms up
  with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what price did
  on these bars; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.bar_horn_bottom(open, high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.horn_bottom (oracle P)
