# One-Day Reversal

Marks a bar that sets a new low for the last `lookback` bars and closes in the
upper half of its own range (+100), or a new high closing in its lower half
(-100).

## Formula

With $M_t = (H_t + L_t)/2$:

$$
\text{bar\_one\_day\_reversal}_t = 100\,[L_t < \min_{t-n \le j < t} L_j \wedge C_t > M_t]
- 100\,[H_t > \max_{t-n \le j < t} H_j \wedge C_t < M_t]
$$

where $n$ is `lookback`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `one_day_reversal`
  (oracle P, `INDICATORS.md` § 5.2).
- The two directions are the oracle's two halves combined its own way: each adds
  its sign and the sum is clipped, so a bar that answers both reads `0`.
- Warm-up is `lookback` bars, the first bar the oracle reads. An `int32` column
  warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what
  price did on these bars; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.bar_one_day_reversal(open, high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.one_day_reversal (oracle P)
