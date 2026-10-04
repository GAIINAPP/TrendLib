# Relative Strength Index

The share of recent movement that was upward, on a scale from 0 to 100. It
averages the bar-to-bar gains and the bar-to-bar losses over `period` changes
and reports the gains as a percentage of the two combined, so a reading near 50
means gains and losses have been of similar size.

## Formula

$$
u_t = \max(x_t - x_{t-1},\, 0) \qquad d_t = \max(x_{t-1} - x_t,\, 0)
$$

$$
\bar{u}_n = \frac{1}{n}\sum_{i=1}^{n} u_i \qquad
\bar{u}_t = \frac{(n-1)\,\bar{u}_{t-1}}{n} + \frac{u_t}{n}, \quad t > n
$$

$$
\mathrm{RSI}_t = 100\,\frac{\bar{u}_t}{\bar{u}_t + \bar{d}_t}
$$

with $\bar{d}$ smoothed exactly as $\bar{u}$, where $x$ is `source` and $n$ is
`period`.

## Conventions

- Lookback is $n$, not $n - 1$: the first value needs $n$ bar-to-bar changes,
  which is $n + 1$ bars. This equals TA-Lib's lookback, which the tests assert.
- Wilder smoothing, seeded with the simple average of the first $n$ gains and
  losses (`CONVENTIONS.md` section 4). An unchanged bar counts as a gain of
  zero.
- When both averages are zero, every bar in the window closed unchanged and the
  index is reported as 0, as TA-Lib reports it. The ratio is undefined there.
- Values agree with ta-lib-python to about 1e-15 relative, inside the 1e-10
  tolerance this project requires, but not bit for bit: TA-Lib's smoothing step
  rounds slightly differently and its exact ordering is not reproducible from
  the published formula.
- Recursive, so early rows still carry a trace of the seed. There is no
  unstable-period setting (D9).
- Leading `NaN` rows are skipped and the seed starts at the first valid bar; a
  `NaN` or infinity after it raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

index = tl.rsi(close, period=14)
```

## References

- J. Welles Wilder Jr., *New Concepts in Technical Trading Systems*, Trend
  Research, 1978, chapter 6.
