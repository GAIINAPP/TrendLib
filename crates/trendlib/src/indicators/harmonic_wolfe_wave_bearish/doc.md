# Bearish Wolfe Wave

Marks a bearish Wolfe wave: five alternating swings inside a narrowing channel,
the fifth reaching beyond the line through the first and third, and the close on
its confirming bar back across that line.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under.

When a swing high $P_5$ is confirmed at bar $t \ge p$, $P_4, P_3, P_2, P_1$ are
chained back from it, each the latest swing of the other side confirmed before
the point after it and within $p$ bars of it. With $\ell$ the line through $P_1$
and $P_3$ (at the bars that confirmed them) read at $t$:

$$
\text{harmonic\_wolfe\_wave\_bearish}_t = -100\,\big[P_3 < P_1 \wedge P_4 > P_2 \wedge P_5 > \ell \wedge C_t < \ell\big]
$$

where $n$ is `pivot_n` and $p$ is `period`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `wolfe_wave_bear`
  with `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
- It reads once, on the bar that confirms the fifth swing.
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
found = tl.harmonic_wolfe_wave_bearish(high, low, close)
```

## References

- Bill Wolfe and Linda Raschke, Wolfe Waves, Technical Analysis of Stocks & Commodities, 1995
- ta-patterns 1.2.1, ta_patterns.chart_patterns.wolfe_wave_bear (oracle P)
