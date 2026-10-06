# Diamond Top

Marks a close below the lower line of a diamond: trendlines over half the window
that were spreading apart half a window ago and are now closing in.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under.

Lines $U$ and $D$ are fitted as for the triangles, but over the swing points
filed in the last $q = \lfloor p/2 \rfloor$ bars; $U'$ and $D'$ are the same fit as
it stood at bar $t - q$.

$$
\text{chart\_diamond\_top}_t = -100\,\big[\text{fitted}_t \wedge \text{fitted}_{t-q}
\wedge s_{U'} > 0 \wedge s_{D'} < 0 \wedge s_D > s_U \wedge C_t < D_t\big]
$$

where a fit is *fitted* when each line has two points and $r^2 \ge 0.5$, $n$ is
`pivot_n` and $p$ is `period`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `diamond_top` with
  `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
- A swing point sits on the bar that confirmed it, `pivot_n` bars after the
  extreme, and lines are drawn through those bars, as in the oracle.
- The reading is repeated on every bar the conditions hold, not only the first.
- A line through swing points of exactly equal price is flat: slope 0, $r^2 =
  1$. The oracle computes a residue of either sign there (`CONVENTIONS.md`
  deviation 9).
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_diamond_top(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- Robert D. Edwards and John Magee, Technical Analysis of Stock Trends, 9th edition, CRC Press, 2007
- ta-patterns 1.2.1, ta_patterns.chart_patterns.diamond_top (oracle P)
