# Volume Weighted Moving Average

A moving average in which each bar counts for as much as it traded. Bars with
little volume move the line less than bars with a lot, so a quiet drift pulls
it less than a busy one.

## Formula

$$
\mathrm{vwma}_t = \frac{\sum_{i=t-n+1}^{t} C_i\,V_i}{\sum_{i=t-n+1}^{t} V_i}
$$

where $n$ is `period`.

## Conventions

- Warm-up is `period - 1` bars.
- A window whose volumes are all zero has no weights to average with, and the
  row is `NaN`. There is no sensible value to stand in for it.
- Both sums are re-added from the window each bar rather than carried forward,
  which a window holding volumes of very different sizes would otherwise drift
  on.
- Not path dependent.

## Example

```python
import trendlib as tl
average = tl.vwma(close, volume, period=30)
```

## References

- TA-Lib, `ta_VWMA.c`.
