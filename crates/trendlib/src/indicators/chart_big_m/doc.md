# Big M

Marks a close below the neckline of a double top drawn at a larger scale: swing
points `pivot_n` 7 bars either side, a 150-bar window and tops at least 15 bars
apart.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under; swing points of
one side filed within $n$ bars of each other are one extreme seen several times
and count once, at the first.

$P_1, P_2$ are the latest two swing highs filed before bar $t$, at bars
$a_1 < a_2$, and $V$ the latest swing low filed before $a_2$, at bar $b > a_1$.

$$
\text{chart\_big\_m}_t = -100\,\Big[a_1 \ge t - p \wedge a_2 - a_1 \ge m
\wedge \frac{|P_1 - P_2|}{\max(|P_1|, |P_2|)} \le \tau \wedge C_t < V\Big]
$$

where $n$ is `pivot_n`, $p$ is `period`, $\tau$ is `tol` and $m$ is `min_sep`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `big_m` with
  `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
- It is `chart_double_top` at the oracle's larger defaults; with the same
  parameters the two agree bar for bar. The oracle's `min_depth` is accepted
  there but never read, and is not offered.
- The reading is repeated on every bar the close stays beyond the neckline while
  the same two tops are the latest.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_big_m(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.big_m (oracle P)
