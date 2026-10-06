# Double Top

Marks a close below the swing low between two recent tops at about the same
price: price has twice failed at one level and has now closed under the trough
between the two attempts.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is placed at.

Swing highs confirmed within $n$ bars of each other are one flat extreme seen
several times and count once, at the first. $P_1, P_2$ are the latest
2 swing highs confirmed before bar $t$, at bars $a_1 < a_2$.
$V$ is the latest swing low confirmed before $a_2$, at bar $b$, and it must lie after the first: $b > a_1$.

$$
\text{chart\_double\_top}_t = \begin{cases}
-100 & a_1 \ge t - p,\ a_2 - a_1 \ge m,\ \frac{|P_1 - P_2|}{\max(|P_1|, |P_2|)} \le \tau,\ C_t < V \\
0 & \text{otherwise}
\end{cases}
$$

where $n$ is `pivot_n`, $p$ is `period`, $\tau$ is `tol`, $m$ is
`min_separation` and $C_t$ is the close.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `double_top` with
  `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.1); `period` is its
  `window`.
- Only swing points confirmed before the current bar count, so a top confirmed
  on the bar of the close is not yet one of them, as in the oracle.
- The reading is repeated on every bar the close stays beyond the neckline while
  the same 2 tops are the latest, not only on the first.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says where
  the close went relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_double_top(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- Robert D. Edwards and John Magee, Technical Analysis of Stock Trends, 9th edition, CRC Press, 2007
- ta-patterns 1.2.1, ta_patterns.chart_patterns.double_top (oracle P)
