# Double Exponential Moving Average

An exponential average with most of its own lag taken back out. Smoothing a
series delays it; smoothing the smoothed series measures roughly how much delay
was added, and subtracting that estimate returns a line that tracks price more
closely than a plain exponential average of the same period.

## Formula

$$
\mathrm{DEMA}_t = 2\,\mathrm{EMA}(x, n)_t - \mathrm{EMA}(\mathrm{EMA}(x, n), n)_t
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Lookback is $2(n - 1)$: two exponential stages in series, each costing
  $n - 1$ warm-up bars.
- The second stage is fed the first stage's output as it appears, so the inner
  average never sees a warm-up row.
- `period = 1` performs no smoothing and returns `source` unchanged.
- Recursive, so early rows still carry a trace of both seeds. There is no
  unstable-period setting (D9).
- Leading `NaN` rows are skipped and the seeds start at the first valid bar; a
  `NaN` or infinity after it raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

smoothed = tl.dema(close, period=30)
```

## References

- Patrick G. Mulloy, "Smoothing Data with Faster Moving Averages", *Technical
  Analysis of Stocks and Commodities*, February 1994.
