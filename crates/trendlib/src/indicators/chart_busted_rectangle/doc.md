# Busted Rectangle

Marks a failed rectangle breakout: after a close through the top of a level
range, a fall of at least `reversal_pct` from the highest close since within
`reversal_bars` bars reads -100; after a close through the bottom, a rise reads
+100.

## Formula

Let $B^+$ be `chart_rectangle`'s upward reading alone and $B^-$ its downward
one, at their defaults. A watch starts on every bar either reads, and ends on its
first qualifying close or after $r$ bars: after $B^+$,
$\big(\max_{s \le j \le k} C_j - C_k\big) / \max_{s \le j \le k} C_j \ge q$ reads $-100$;
after $B^-$, $\big(C_k - \min_{s \le j \le k} C_j\big) / \big|\min_{s \le j \le k} C_j\big| \ge q$
reads $+100$. The two add and clip, so a bar answering both reads 0. $r$ is
`reversal_bars` and $q$ is `reversal_pct`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `busted_rectangle`
  with `mode="confirmed"` (oracle P, `INDICATORS.md` § 5.2).
- The two directions of `chart_rectangle` are watched apart, as the oracle's
  `rectangle_bottom` and `rectangle_top` are: a bar can break out both ways only
  where the lines cross, and then both start a watch.
- Warm-up is `chart_rectangle`'s. An `int32` column warms up with `0`, not `NaN`
  (`CONVENTIONS.md` § 2), and the output says what price did after the pattern;
  it names no price level.
- Path dependent, as its base pattern is.

## Example

```python
import trendlib as tl
found = tl.chart_busted_rectangle(high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.busted_rectangle (oracle P)
