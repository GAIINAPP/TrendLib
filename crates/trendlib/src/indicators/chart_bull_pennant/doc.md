# Bull Pennant

Marks a close above the upper line of a pennant: a sharp rise (the pole), then
a short pause between a falling line over the swing highs and a rising line
under the swing lows, and now a close above the pause.

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
\Delta_t = \frac{C_{t-p-1} - C_{t-p-1-q}}{C_{t-p-1-q}}
$$

$$
\text{chart\_bull\_pennant}_t = \begin{cases}
+100 & \Delta_t \ge \mu,\ \text{fitted},\ s_U < 0 < s_D,\ C_t > U_t \\
0 & \text{otherwise}
\end{cases}
$$

where $n$ is `pivot_n`, $p$ is `period`, $q$ is `pole_bars`, $\mu$ is `min_pole` and $C_t$ is the close; the lines are fitted over the pennant's `period` bars.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `pennant_bull`
  with `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.1); `period` is its
  `window`.
- A pole of exactly `min_pole` counts, and a rise is asked about before a fall,
  as the oracle asks.
- The first value is on row `pole_bars + period + 1`, the first with a whole
  pole behind it. The oracle reads one row earlier, where its pole would start
  before the first bar and NumPy hands it the last bar of the input instead
  (`CONVENTIONS.md` deviation 8).
- A swing point sits on the bar that confirmed it, `pivot_n` bars after the
  extreme, and the lines are drawn through those bars. That is the oracle's
  convention, and it reads a line `pivot_n` bars later than the extremes it
  passes through.
- The reading is repeated on every bar the conditions hold, not only the first.
- A line through swing points of exactly equal price is flat: slope 0, $r^2 =
  1$. The oracle computes a residue of either sign there (`CONVENTIONS.md`
  deviation 9).
- Warm-up is `pole_bars + period + 1` bars. An `int32` column warms up with `0`,
  not `NaN` (`CONVENTIONS.md` § 2), and the output says where the close went
  relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_bull_pennant(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- Robert D. Edwards and John Magee, Technical Analysis of Stock Trends, 9th edition, CRC Press, 2007
- ta-patterns 1.2.1, ta_patterns.chart_patterns.pennant_bull (oracle P)
