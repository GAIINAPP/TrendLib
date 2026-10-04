# Exponential Moving Average

A weighted mean in which each older bar counts less than the one after it, by a
constant factor. It follows a change in the series sooner than a simple moving
average of the same period, and it never fully discards an old bar: every past
value keeps a small weight forever.

## Formula

$$
k = \frac{2}{n + 1} \qquad
\mathrm{EMA}_{n-1} = \frac{1}{n}\sum_{i=0}^{n-1} x_i
$$

$$
\mathrm{EMA}_t = \mathrm{EMA}_{t-1} + k\,(x_t - \mathrm{EMA}_{t-1}), \quad t \ge n
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Seeded with the simple mean of the first $n$ values, as TA-Lib seeds it, so
  lookback is $n - 1$.
- Advanced as `prev + (x - prev) * k`. The algebraically equal form
  `x*k + prev*(1-k)` rounds differently and drifts from the oracle by a few ULP
  (`CONVENTIONS.md` section 1).
- Recursive, so the value at any bar still carries a trace of the seed. There is
  no unstable-period setting (D9); discard extra leading rows yourself if you
  want values independent of where the series starts.
- `period = 1` performs no smoothing and returns `source` unchanged.
- Leading `NaN` rows are skipped and the seed starts at the first valid bar; a
  `NaN` or infinity after it raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

smoothed = tl.ema(close, period=30)
```

## References

- John J. Murphy, *Technical Analysis of the Financial Markets*, New York
  Institute of Finance, 1999, chapter 9.
