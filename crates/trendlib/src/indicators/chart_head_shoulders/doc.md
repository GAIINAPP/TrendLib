# Head and Shoulders

Marks the first close below the neckline of a head and shoulders: three swing
highs, the middle one highest and the outer two at about the same height, with
a line drawn through the two troughs between them.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is placed at.

When a swing high $R$ (the right shoulder) is confirmed at bar $r$:

- the head $H$, confirmed at bar $h$, is the highest swing high confirmed in
  $[r - p,\, r)$, the first if two tie, and $H > R$;
- the left shoulder $S$, confirmed at bar $s$, is the latest swing high in that
  window before the head with $h - s \ge m$, and $r - h \ge m$;
- the shoulders match: $|S - R| / \max(S, R) \le \tau$;
- the neckline $N$ is the line through the latest swing low confirmed in
  $(s, h)$ and the latest confirmed in $(h, r)$.

$$
\text{chart\_head\_shoulders}_t = \begin{cases}
-100 & t \text{ is the first bar in } (r,\, r + p) \text{ with } C_t < N_t \\
0 & \text{otherwise}
\end{cases}
$$

where $n$ is `pivot_n`, $p$ is `period`, $\tau$ is `shoulder_tol`, $m$ is
`min_separation` and $C_t$ is the close.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `hs_top` with
  `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.1); `period` is its
  `window`.
- Each right shoulder arms its own neckline, which waits `period` bars and is
  spent by the first close through it; the reading is that one bar, not every
  bar after it.
- A swing point sits on the bar that confirmed it, `pivot_n` bars after the
  extreme, and the neckline is drawn through those bars, as in the oracle.
- Warm-up is `pivot_n + 2 * min_separation + 1` bars, the earliest a neckline
  can be crossed: the left shoulder confirmed `pivot_n` bars in, head and right
  shoulder `min_separation` apart after it, and one bar more. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says where
  the close went relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_head_shoulders(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- Robert D. Edwards and John Magee, Technical Analysis of Stock Trends, 9th edition, CRC Press, 2007
- ta-patterns 1.2.1, ta_patterns.chart_patterns.hs_top (oracle P)
