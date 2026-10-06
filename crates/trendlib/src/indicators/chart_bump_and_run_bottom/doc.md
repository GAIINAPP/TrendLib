# Bump-and-Run Reversal Bottom

Marks the run after a bump: a steady lead-in line through the swing highs, a
bump at least `bump_factor` times as steep, and a close back above the lead-in
line extended to this bar.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under.

$G$ is the least-squares line through the swing highs filed in
$[t - b - a,\, t - b - 1]$ (the lead-in), read at bar $t$; $B$ the line through
those filed in $[t - b,\, t]$ (the bump).

$$
\text{chart\_bump\_and\_run\_bottom}_t = 100\,\big[|G| \ge 2 \wedge r^2_G \ge 0.6 \wedge |B| \ge 2
\wedge s_G < 0 \wedge s_B \le k\,s_G \wedge C_t > G_t\big]
$$

where $|\cdot|$ counts a line's points, $a$ is `lead_window`, $b$ is
`bump_window`, $k$ is `bump_factor` and $n$ is `pivot_n`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's
  `bump_and_run_bottom` with `mode="confirmed"` (oracle P, `INDICATORS.md` §
  5.2).
- The lead-in fit needs $r^2 \ge 0.6$, the oracle's constant.
- A swing point sits on the bar that confirmed it, `pivot_n` bars after the
  extreme, and lines are drawn through those bars, as in the oracle.
- The reading is repeated on every bar the conditions hold, not only the first.
- A line through swing points of exactly equal price is flat: slope 0, $r^2 =
  1$. The oracle computes a residue of either sign there (`CONVENTIONS.md`
  deviation 9).
- Warm-up is `lead_window + bump_window` bars, the first bar the oracle reads.
  An `int32` column warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the
  output says what price did relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_bump_and_run_bottom(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- Thomas N. Bulkowski, Bump-and-Run Reversals, Technical Analysis of Stocks & Commodities, 1997
- ta-patterns 1.2.1, ta_patterns.chart_patterns.bump_and_run_bottom (oracle P)
