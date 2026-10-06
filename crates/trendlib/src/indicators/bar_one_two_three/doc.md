# 1-2-3 Reversal

Marks the 1-2-3 reversal: two swing lows in the window, the second higher, with
a swing high between them, and now a close above that swing high (+100); or the
mirror at a top (-100).

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and is filed under that bar with the price **of that
bar**: $L_{k+n}$ for a swing low, $H_{k+n}$ for a swing high.

Among the swing lows filed in $[t - p,\, t)$ let $a_1 < a_3$ be the last two, and
$a_2$ the first swing high filed after $a_1$:

$$
\text{up}_t = a_2 < a_3 \wedge L_{a_3} > L_{a_1} \wedge C_t > H_{a_2}
$$

$\text{down}_t$ is the mirror on swing highs, and
$\text{bar\_one\_two\_three}_t = 100\,[\text{up}_t] - 100\,[\text{down}_t]$,
where $n$ is `pivot_n` and $p$ is `period`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `one_two_three`
  (oracle P, `INDICATORS.md` § 5.2); `period` is its `window`.
- A swing point is priced at the bar that confirmed it, not at the extreme; that
  is the oracle's reading and TrendLib keeps it so the oracle can check it.
- The oracle's `tol` parameter changes nothing in it and is not offered.
- The two directions are the oracle's two halves combined its own way: each adds
  its sign and the sum is clipped, so a bar that answers both reads `0`.
- Warm-up is `period` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did on these bars; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are.

## Example

```python
import trendlib as tl
found = tl.bar_one_two_three(open, high, low, close)
```

## References

- Jack D. Schwager, Technical Analysis, Wiley, 1996
- ta-patterns 1.2.1, ta_patterns.chart_patterns.one_two_three (oracle P)
