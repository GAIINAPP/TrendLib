# Double Top, Eve and Eve

Marks a close below the swing low between two tops at about the same price,
where the first top is Eve (rounded) and the second Eve (rounded): Bulkowski's
Eve and Eve double top.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under; swing points of
one side filed within $n$ bars of each other are one extreme seen several times
and count once, at the first.

$P_1, P_2$ are the latest two swing highs filed before bar $t$, at bars
$a_1 < a_2$, and $V$ the latest swing low filed before $a_2$, at bar $b > a_1$.

$$
\text{chart\_double\_top\_eve\_eve}_t = -100\,\Big[a_1 \ge t - p \wedge a_2 - a_1 \ge m
\wedge \frac{|P_1 - P_2|}{\max(|P_1|, |P_2|)} \le \tau \wedge C_t < V \wedge \text{shapes}\Big]
$$

where *shapes* asks that the first top be Eve (rounded) and the second
Eve (rounded). A top is Adam when, over its own bar and the three either
side ($W$, highs), $g = \frac{\max W - \bar W}{\max W - \min W} \ge 0.37$, and Eve otherwise
(also when $W$ is flat). $n$ is `pivot_n`, $p$ is `period`, $\tau$ is `tol`, $m$
is `min_separation`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's
  `double_top_eve_eve` with `mode="confirmed"` (oracle P, `INDICATORS.md` §
  5.2); `period` is its `window`.
- The grade, its seven-bar window and the 0.37 cut are the oracle's; they are
  not parameters.
- `pivot_n` starts at 3 so a top's three bars on the right are known when its
  swing is confirmed; with less the oracle grades it with bars that have not
  happened yet.
- Without the shape test this is `chart_double_top`; every Eve and Eve reading
  is one of its readings.
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
found = tl.chart_double_top_eve_eve(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.double_top_eve_eve (oracle P)
