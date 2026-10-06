# Bullish Bat

Marks a bullish Bat: an XABCD swing sequence whose legs keep the Bat's Fibonacci
proportions, AB about 0.441 of XA and AD about 0.886 of it, read when the close
on D's confirming bar is above D.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under.

When a swing low $D$ is confirmed at bar $t \ge p$, the earlier points
are chained back from it: $C$ is the latest swing high confirmed before $D$, $B$
the latest swing low before $C$, $A$ the latest swing high before $B$, and $X$ the latest swing low before $A$, each
within $p$ bars of the point after it, or there is no pattern.
With legs measured in the direction the shape runs, $XA = (A - X)$,
$AB = (A - B)$, $BC = (C - B)$, $CD = (C - D)$, $AD = (A - D)$, all of the
first four positive, and $|r/m - 1| \le \tau$ written $r \approx m$:

$$
\frac{AB}{XA} \approx 0.441 \qquad 0.382 \le \frac{BC}{AB} \le 0.886 \qquad
1.618 \le \frac{CD}{BC} \le 2.618 \qquad \frac{|AD|}{XA} \approx 0.886
$$

and `harmonic_bat_bullish` reads +100 at $t$ when all hold and $C_t > D$, where $n$ is
`pivot_n`, $p$ is `period`, $\tau$ is `fib_tol` and $C_t$ is the close.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `bat_bull` with
  `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
- It reads once, on the bar that confirms D.
- The ratios are the oracle's constants for the Bat (the midpoint 0.441 of the
  0.382 to 0.5 band for AB); only the tolerance is a parameter.
- Each link is only the latest swing of its side before the next point; an older
  one is not tried.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.harmonic_bat_bullish(high, low, close)
```

## References

- Scott M. Carney, Harmonic Trading, Volume One, FT Press, 2010
- ta-patterns 1.2.1, ta_patterns.chart_patterns.bat_bull (oracle P)
