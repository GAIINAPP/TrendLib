# Ascending Triangle

Marks a close above a flat line drawn over the recent swing highs while a
rising line drawn under the swing lows closes in on it: buyers have repeatedly
met the same ceiling from ever higher floors, and price has now closed through
it.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is placed at.

Through the swing highs placed in $[t - p,\, t]$ an ordinary least-squares
line $U$ is fitted: slope $s_U$ in price per bar, value $U_t$ at bar $t$ and
coefficient of determination $r^2_U$. $D$ is fitted through the swing lows the
same way. The two are *fitted* when each has at least two points and

$$
r^2_U \ge 0.5 \quad\text{and}\quad r^2_D \ge 0.5
$$

$$
\text{chart\_ascending\_triangle}_t = \begin{cases}
+100 & \text{fitted},\ |s_U| \le f,\ s_D > 0,\ s_D > s_U,\ C_t > U_t \\
0 & \text{otherwise}
\end{cases}
$$

where $n$ is `pivot_n`, $p$ is `period`, $f$ is `flat_tol` and $C_t$ is the close.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's
  `ascending_triangle` with `mode="confirmed"` (oracle P, `INDICATORS.md` §
  5.1); `period` is its `window`.
- A swing point sits on the bar that confirmed it, `pivot_n` bars after the
  extreme, and the lines are drawn through those bars. That is the oracle's
  convention, and it reads a line `pivot_n` bars later than the extremes it
  passes through.
- The reading is repeated on every bar the conditions hold, not only the first.
- A line through swing points of exactly equal price is flat: slope 0, $r^2 =
  1$. The oracle computes a residue of either sign there (`CONVENTIONS.md`
  deviation 9).
- `flat_tol` is a slope in price per bar, not a fraction, as the oracle has it:
  0.02 is nearly level on a price of 100 and far steeper than any flat line on a
  price of 1. Scale it with the instrument.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says where
  the close went relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_ascending_triangle(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- Robert D. Edwards and John Magee, Technical Analysis of Stock Trends, 9th edition, CRC Press, 2007
- ta-patterns 1.2.1, ta_patterns.chart_patterns.ascending_triangle (oracle P)
