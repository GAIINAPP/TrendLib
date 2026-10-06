# Three Peaks

Marks three falling peaks: the last three swing highs in the window each lower
than the one before, and a close below the lowest low of the bars between the
first and the third.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under; swing points of
one side filed within $n$ bars of each other are one extreme seen several times
and count once, at the first.

$P_1, P_2, P_3$ are the last three swing highs filed in $[t - p,\, t)$, at bars
$a_1 < a_2 < a_3$:

$$
\text{chart\_three\_peaks}_t = -100\,\Big[a_2 - a_1 \ge m \wedge a_3 - a_2 \ge m
\wedge P_1 > P_2 > P_3 \wedge C_t < \min_{a_1 \le j \le a_3} L_j\Big]
$$

where $n$ is `pivot_n`, $p$ is `period` and $m$ is `min_separation`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `three_peaks` with
  `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2); `period` is its
  `window`.
- The lowest low is taken over the bars from the first swing's confirmation to
  the third's, as the oracle takes it, not from extreme to extreme.
- The oracle's `tol` is accepted but never read, and is not offered.
- The reading is repeated on every bar the conditions hold, not only the first.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_three_peaks(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.three_peaks (oracle P)
