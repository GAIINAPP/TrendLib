# Triple Exponential Moving Average

The same lag-removal idea as the double exponential average, carried one stage
further. Three nested exponential averages are combined so that more of the
delay smoothing introduces is cancelled, giving a line that turns sooner again,
at the cost of reacting more to single bars.

## Formula

$$
e^{(1)} = \mathrm{EMA}(x, n), \quad
e^{(2)} = \mathrm{EMA}(e^{(1)}, n), \quad
e^{(3)} = \mathrm{EMA}(e^{(2)}, n)
$$

$$
\mathrm{TEMA}_t = 3\,e^{(1)}_t - 3\,e^{(2)}_t + e^{(3)}_t
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Lookback is $3(n - 1)$: three exponential stages in series.
- Each stage is fed the previous stage's output as it appears, so no stage sees
  a warm-up row.
- `period = 1` performs no smoothing and returns `source` unchanged.
- Recursive, so early rows still carry a trace of all three seeds (D9).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

smoothed = tl.tema(close, period=30)
```

## References

- Patrick G. Mulloy, "Smoothing Data with Faster Moving Averages", *Technical
  Analysis of Stocks and Commodities*, February 1994.
