# Pipe Top

Marks a pipe top: two adjacent bars, each with a range more than 1.2 times the
14-bar average true range, whose highs are within `height_tol` of each other,
the second closing below its open.

## Formula

With $A_t$ the mean of the true range over bars $t-13 \ldots t$:

$$
\text{bar\_pipe\_top}_t = -100\,\Big[A_t > 0 \wedge H_{t-1} - L_{t-1} > 1.2 A_t \wedge H_t - L_t > 1.2 A_t
\wedge \frac{|H_t - H_{t-1}|}{H_t} < \tau \wedge C_t < O_t\Big]
$$

where $\tau$ is `height_tol`.

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `pipe_top` (oracle
  P, `INDICATORS.md` § 5.2).
- The 14 bars of the average and the factor 1.2 are the oracle's constants, not
  parameters. The first bar's true range uses its own close as the previous one.
- The average is summed afresh from its 14 bars each time (`CONVENTIONS.md` §
  1); the oracle takes it as a difference of running totals, and the two can
  differ only where a range sits exactly on the threshold.
- Warm-up is 13 bars, the first with a 14-bar average, the first bar the oracle
  reads. An `int32` column warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2),
  and the output says what price did on these bars; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.bar_pipe_top(open, high, low, close)
```

## References

- Thomas N. Bulkowski, Encyclopedia of Chart Patterns, 2nd edition, Wiley, 2005
- ta-patterns 1.2.1, ta_patterns.chart_patterns.pipe_top (oracle P)
