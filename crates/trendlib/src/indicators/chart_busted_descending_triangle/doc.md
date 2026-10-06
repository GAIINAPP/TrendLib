# Busted Descending Triangle

Marks a failed descending triangle: after `chart_descending_triangle` reads,
closes rise at least `reversal_pct` from the lowest close since, within
`reversal_bars` bars. It reads +100 on the first such bar.

## Formula

Let $B$ be `chart_descending_triangle` at its defaults. For every bar $s$ where $B_s \ne 0$, and
each later bar $k$ in $(s,\, s + r]$ until one qualifies:

$$
\text{busted}_k = \frac{C_k - \min_{s \le j \le k} C_j}{\big|\min_{s \le j \le k} C_j\big|} \ge q
$$

and `chart_busted_descending_triangle` reads +100 on the first such $k$ of each $s$, where $r$ is
`reversal_bars` and $q$ is `reversal_pct`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's
  `busted_desc_triangle` with `mode="confirmed"` (oracle P, `INDICATORS.md` §
  5.2).
- The base pattern is `chart_descending_triangle` at its own defaults, which the
  oracle's busted function uses too; its parameters are not offered here.
- Every bar the base pattern reads starts its own watch, so a run of readings
  starts a run of watches, as in the oracle; each ends at its first qualifying
  close or after `reversal_bars` bars.
- Warm-up is `chart_descending_triangle`'s. An `int32` column warms up with `0`,
  not `NaN` (`CONVENTIONS.md` § 2), and the output says what price did after the
  pattern; it names no price level.
- Path dependent, as its base pattern is.

## Example

```python
import trendlib as tl
found = tl.chart_busted_descending_triangle(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- Thomas N. Bulkowski, Trading Classic Chart Patterns, Wiley, 2002
- ta-patterns 1.2.1, ta_patterns.chart_patterns.busted_desc_triangle (oracle P)
