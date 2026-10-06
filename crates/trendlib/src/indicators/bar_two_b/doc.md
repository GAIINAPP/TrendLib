# 2B Reversal

Marks a failed breakout: one bar reaches beyond the highest high (lowest low) of
the bars before it by more than `tol`, and the next bar closes back inside that
extreme. Reads -100 for a failed high and +100 for a failed low.

## Formula

With $P^H_t = \max_{t-n-1 \le j \le t-2} H_j$ and $P^L_t = \min_{t-n-1 \le j \le t-2} L_j$:

$$
\text{up}_t = L_{t-1} < P^L_t (1 - \tau) \;\wedge\; C_t > P^L_t \qquad
\text{down}_t = H_{t-1} > P^H_t (1 + \tau) \;\wedge\; C_t < P^H_t
$$

$$
\text{bar\_two\_b}_t = 100\,[\text{up}_t] - 100\,[\text{down}_t]
$$

where $n$ is `lookback`, $\tau$ is `tol` and $[\cdot]$ is 1 when its condition holds.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `two_b` (oracle P,
  `INDICATORS.md` § 5.2).
- The two directions are the oracle's two halves combined its own way: each adds
  its sign and the sum is clipped, so a bar that answers both reads `0`.
- Warm-up is `lookback + 1` bars, the first bar the oracle reads. An `int32`
  column warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output
  says what price did on these bars; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.bar_two_b(open, high, low, close)
```

## References

- Victor Sperandeo, Trader Vic - Methods of a Wall Street Master, Wiley, 1991
- ta-patterns 1.2.1, ta_patterns.chart_patterns.two_b (oracle P)
