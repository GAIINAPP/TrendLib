# Island Top

Marks an island top: a gap up within the last `max_island_bars` bars, and now a
bar that gaps down below both the bar before it and the bar that gapped up,
closing below it.

## Formula

Let $e = t - k$ for the smallest $k$ in $1 \ldots m - 1$ with $L_e > H_{e-1}$
(a gap up into the island). Then

$$
\text{chart\_island\_top}_t = -100\,\big[H_t < L_{t-1} \wedge H_t < L_e \wedge C_t < L_e\big]
$$

and 0 when there is no such $k$, where $m$ is `max_island_bars`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `island_top` with
  `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2).
- Only the most recent gap into the island is tested, as in the oracle; an older
  one behind it is not looked for.
- Warm-up is `max_island_bars + 1` bars, the first bar the oracle reads. An
  `int32` column warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the
  output says what price did relative to the shape; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.chart_island_top(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.island_top (oracle P)
