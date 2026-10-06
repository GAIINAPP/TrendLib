# Wide-Ranging Day

Marks a bar whose range is more than `factor` times the average range of the
`lookback` bars before it, reading +100 when it closes in its upper half and
-100 in its lower half.

## Formula

With $R_t = H_t - L_t$, $\bar R_t = \frac1n \sum_{j=t-n}^{t-1} R_j$ and $M_t = (H_t + L_t)/2$:

$$
\text{bar\_wide\_ranging\_day}_t = 100\,[R_t > k \bar R_t \wedge C_t > M_t] - 100\,[R_t > k \bar R_t \wedge C_t < M_t]
$$

where $n$ is `lookback` and $k$ is `factor`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `wide_ranging_day`
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
found = tl.bar_wide_ranging_day(open, high, low, close)
```

## References

- Laurence A. Connors and Linda Bradford Raschke, Street Smarts, M. Gordon Publishing, 1995
- ta-patterns 1.2.1, ta_patterns.chart_patterns.wide_ranging_day (oracle P)
