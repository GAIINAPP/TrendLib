# Bearish AB=CD

Marks a bearish AB=CD: the swing from C to D as long as the swing from A to B
(within `fib_tol`), C having retraced 0.382 to 0.886 of A to B, and the close on
D's confirming bar below D.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under.

When a swing high $D$ is confirmed at bar $t \ge p$, the earlier points
are chained back from it: $C$ is the latest swing low confirmed before $D$, $B$
the latest swing high before $C$, $A$ the latest swing low before $B$, each
within $p$ bars of the point after it, or there is no pattern.
With legs measured in the direction the shape runs, $AB = -(A - B)$,
$CD = -(C - D)$:

$$
\text{harmonic\_abcd\_bearish}_t = -100\,\Big[AB > 0 \wedge CD > 0
\wedge 0.382 \le \frac{|C - B|}{AB} \le 0.886
\wedge \Big|\frac{CD}{AB} - 1\Big| \le \tau \wedge C_t < D\Big]
$$

where $n$ is `pivot_n`, $p$ is `period`, $\tau$ is `fib_tol` and $C_t$ is the close.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `abcd_bear` with
  `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
- It reads once, on the bar that confirms D.
- Each link is only the latest swing of its side before the next point; an older
  one is not tried.
- The 0.382 to 0.886 band is the oracle's constant, not a parameter.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.harmonic_abcd_bearish(high, low, close)
```

## References

- Scott M. Carney, Harmonic Trading, Volume One, FT Press, 2010
- ta-patterns 1.2.1, ta_patterns.chart_patterns.abcd_bear (oracle P)
