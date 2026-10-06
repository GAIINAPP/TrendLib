# Key Reversal

Marks a bar that sets a new low for the last `lookback` bars yet closes above
the previous close (+100), or a new high that closes below it (-100).

## Formula

$$
\text{up}_t = L_t < \min_{t-n \le j < t} L_j \;\wedge\; C_t > C_{t-1} \qquad
\text{down}_t = H_t > \max_{t-n \le j < t} H_j \;\wedge\; C_t < C_{t-1}
$$

$$
\text{bar\_key\_reversal}_t = 100\,[\text{up}_t] - 100\,[\text{down}_t]
$$

where $n$ is `lookback`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `key_reversal`
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
found = tl.bar_key_reversal(open, high, low, close)
```

## References

- Jack D. Schwager, Technical Analysis, Wiley, 1996
- ta-patterns 1.2.1, ta_patterns.chart_patterns.key_reversal (oracle P)
