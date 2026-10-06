# Inverted Dead-Cat Bounce

Marks a shallow pullback after a sharp rise: closes rose at least `rise_pct`
over `period` bars, and the close now gives back between 5 percent and
`pullback_pct` of that rise.

## Formula

With $h = \lfloor w/2 \rfloor$, $S = C_{t-h-w}$ and $P = \max_{t-h \le j < t} C_j$:

$$
\text{bar\_inverted\_dead\_cat\_bounce}_t = 100\,\Big[\frac{P - S}{S} \ge r
\wedge 0.05 < \frac{P - C_t}{P - S} < b\Big]
$$

where $w$ is `period`, $r$ is `rise_pct` and $b$ is `pullback_pct`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's
  `dead_cat_bounce_inv` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
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
found = tl.bar_inverted_dead_cat_bounce(open, high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.dead_cat_bounce_inv (oracle P)
