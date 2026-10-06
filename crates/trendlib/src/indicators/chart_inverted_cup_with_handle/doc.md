# Inverted Cup with Handle

Marks a close below the floor of an inverted cup: a rounded top spanning
`cup_window` bars, a short bounce that stays near its floor, and now a close
beneath that floor.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under.

The cup is the swing points confirmed in $[t - q - w,\, t - w)$ and the
handle the closes of $[t - w,\, t)$. With $P$ the highest swing high and $F$ the
lowest swing low of the cup, $d = P - F$ and $m = \min_{t-w \le j < t} C_j$:

$$
\text{chart\_inverted\_cup\_with\_handle}_t = \begin{cases}
-100 & \dfrac{d}{P} \ge 0.05,\ m \le 1.05\,F,\ \dfrac{C_t - m}{d} \le \rho,\ C_t < F \\
0 & \text{otherwise}
\end{cases}
$$

where $n$ is `pivot_n`, $q$ is `cup_window`, $w$ is `handle_window`, $\rho$ is
`max_handle_retrace` and $C_t$ is the close.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's
  `inverted_cup_with_handle` with `mode="confirmed"` (oracle P, `INDICATORS.md`
  § 5.2); `period` is its `window`.
- A swing point belongs to the cup if the bar that confirmed it, `pivot_n` bars
  after the extreme, falls in the cup's span; the oracle files it the same way.
- The depth test (5 percent of the cup's top) and the handle's 5 percent band
  are the oracle's fixed constants, not parameters.
- The reading is repeated on every bar the conditions hold, not only the first.
- Warm-up is `cup_window + handle_window` bars, the first bar the oracle reads.
  An `int32` column warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the
  output says what price did relative to the shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_inverted_cup_with_handle(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- William J. O'Neil, How to Make Money in Stocks, McGraw-Hill, 1988
- ta-patterns 1.2.1, ta_patterns.chart_patterns.inverted_cup_with_handle (oracle P)
