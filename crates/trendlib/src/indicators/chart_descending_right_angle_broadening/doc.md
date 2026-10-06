# Descending Right-Angled Broadening

Marks a close above a rising line over the recent swing highs while the line
under the swing lows stays flat: a right-angled broadening formation, descending
in the oracle's naming.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under.

Through the swing highs filed in $[t - p,\, t]$ an ordinary least-squares
line $U$ is fitted: slope $s_U$, value $U_t$ at bar $t$, fit $r^2_U$; $D$ through
the swing lows the same way. The two are *fitted* when each has at least two
points and $r^2_U \ge 0.5$, $r^2_D \ge 0.5$.

$$
\text{chart\_descending\_right\_angle\_broadening}_t = 100\,\big[\text{fitted} \wedge s_U > 0 \wedge |s_D| \le f \wedge C_t > U_t\big]
$$

where $n$ is `pivot_n`, $p$ is `period`, $f$ is `flat_tol` and $C_t$ is the close.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's
  `right_angle_broadening_desc` with `mode="confirmed"` (oracle P,
  `INDICATORS.md` § 5.2); `period` is its `window`.
- A swing point sits on the bar that confirmed it, `pivot_n` bars after the
  extreme, and lines are drawn through those bars, as in the oracle.
- The reading is repeated on every bar the conditions hold, not only the first.
- A line through swing points of exactly equal price is flat: slope 0, $r^2 =
  1$. The oracle computes a residue of either sign there (`CONVENTIONS.md`
  deviation 9).
- `flat_tol` is a slope in price per bar, as the oracle has it; scale it with
  the instrument.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_descending_right_angle_broadening(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- Robert D. Edwards and John Magee, Technical Analysis of Stock Trends, 9th edition, CRC Press, 2007
- ta-patterns 1.2.1, ta_patterns.chart_patterns.right_angle_broadening_desc (oracle P)
