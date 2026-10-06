# Measured Move Up

Marks a measured move up: two rising legs of about the same length, read on the
bar that confirms the second leg's end.

## Formula

Bar $k$ is a swing high when $H_k \ge H_j$ for every bar $j$ with
$|j - k| \le n$, and a swing low when $L_k \le L_j$ for every such bar. It is
known at bar $k + n$, and that is the bar it is filed under.

For consecutive swing highs $H_1$ then $H_2$, the latter confirmed at bar $t$: $L_1$
is the latest swing low confirmed before $H_1$, and $L_2$ the lowest confirmed
strictly between them. With legs $\ell_1 = H_1 - L_1$ and $\ell_2 = H_2 - L_2$:

$$
\text{chart\_measured\_move\_up}_t = 100\,\Big[\ell_1 > 0 \wedge \ell_2 > 0
\wedge \frac{|\ell_2 - \ell_1|}{\ell_1} < \lambda \wedge C_t > H_1\Big]
$$

where $n$ is `pivot_n` and $\lambda$ is `leg_tol`; every other bar reads 0.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `measured_move_up`
  with `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2).
- It reads only the bar that confirms the second high, once.
- There is no window: consecutive swing highs are paired however far apart, as
  in the oracle, whose `window` parameter it never reads.
- A swing low confirmed on the same bar as a swing high lies neither before nor
  between, since both tests are strict.
- Warm-up is `pivot_n + 3` bars, the earliest a second leg can end, the first
  bar the oracle reads. An `int32` column warms up with `0`, not `NaN`
  (`CONVENTIONS.md` § 2), and the output says what price did relative to the
  shape; it names no price level.
- Path dependent: a swing point near the start of the input is judged on the
  bars there are, so a batch over a slice can find swing points the full series
  does not have until a window has passed.

## Example

```python
import trendlib as tl
found = tl.chart_measured_move_up(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.measured_move_up (oracle P)
