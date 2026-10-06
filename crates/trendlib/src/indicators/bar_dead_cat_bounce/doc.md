# Dead-Cat Bounce

Marks a weak bounce after a sharp fall: closes fell at least `drop_pct` over
`period` bars, and the close now recovers between 5 percent and `bounce_pct` of
that fall.

## Formula

With $h = \lfloor w/2 \rfloor$, $S = C_{t-h-w}$ and $T = \min_{t-h \le j < t} C_j$:

$$
\text{bar\_dead\_cat\_bounce}_t = -100\,\Big[\frac{S - T}{S} \ge d
\wedge 0.05 < \frac{C_t - T}{S - T} < b\Big]
$$

where $w$ is `period`, $d$ is `drop_pct` and $b$ is `bounce_pct`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `dead_cat_bounce`
  (oracle P, `INDICATORS.md` § 5.2); `period` is its `window`.
- The move is measured from the close `period + period / 2` bars back to the
  extreme close of the last `period / 2` bars; the 5 percent floor on the bounce
  is the oracle's constant.
- Warm-up is `2 * period` bars, the first the oracle reads, the first bar the
  oracle reads. An `int32` column warms up with `0`, not `NaN` (`CONVENTIONS.md`
  § 2), and the output says what price did on these bars; it names no price
  level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.bar_dead_cat_bounce(open, high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.dead_cat_bounce (oracle P)
