# Wilder Smoothed Moving Average

An exponential-style average that gives a new bar a weight of one over the
period. It smooths more slowly than an exponential average of the same length
and is the averaging Wilder used throughout his own indicators.

## Formula

$$
\mathrm{RMA}_n = \frac{1}{n}\sum_{i=1}^{n} x_i \qquad \mathrm{RMA}_t = \frac{(n-1)\,\mathrm{RMA}_{t-1}}{n} + \frac{x_t}{n}
$$

## Conventions

- Lookback is $n - 1$, seeded with the simple mean of the first $n$ values.
- Recursive, so early rows still carry a trace of the seed; there is no
  unstable-period setting (D9).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

smoothed = tl.rma(close, period=30)
```

## References

- J. Welles Wilder Jr., New Concepts in Technical Trading Systems, Trend Research, 1978
