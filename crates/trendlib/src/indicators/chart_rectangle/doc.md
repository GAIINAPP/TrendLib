# Rectangle

Marks a close through either side of a level range: a flat line over the
recent swing highs and a flat line under the swing lows. Reads +100 for a
close above the upper line and -100 for one below the lower line.

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
\text{chart\_rectangle}_t = \begin{cases}
+100 & \text{fitted},\ |s_U| \le f,\ |s_D| \le f,\ C_t > U_t \\
-100 & \text{fitted},\ |s_U| \le f,\ |s_D| \le f,\ C_t < D_t \\
0 & \text{otherwise}
\end{cases}
$$

where $n$ is `pivot_n`, $p$ is `period`, $f$ is `flat_tol` and $C_t$ is the close.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `rectangle_bottom`
  and `rectangle_top` with `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.1);
  `period` is its `window`.
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
- The two directions are the oracle's `rectangle_bottom` (+100) and
  `rectangle_top` (-100), whose confirmed readings ask nothing of the trend
  before the range. A close is tested against the upper line first.
- The fit asks for $r^2 \ge 0.5$ on both lines, and a level line through
  scattered points explains little of their spread, so this reads ranges whose
  swing points lie close to a line: a drift within `flat_tol` passes, noise
  around a level mostly does not.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says where
  the close went relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_rectangle(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- Robert D. Edwards and John Magee, Technical Analysis of Stock Trends, 9th edition, CRC Press, 2007
- ta-patterns 1.2.1, ta_patterns.chart_patterns.rectangle_bottom and ta_patterns.chart_patterns.rectangle_top (oracle P)
