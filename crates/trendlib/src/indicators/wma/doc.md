# Weighted Moving Average

A mean of the last `period` values in which each bar counts in proportion to how
recent it is: the newest weighs `period`, the one before it `period - 1`, and so
on down to 1. It turns sooner than a simple moving average of the same length
and drops a bar completely once it leaves the window.

## Formula

$$
\mathrm{WMA}(x, n)_t = \frac{\sum_{i=0}^{n-1} (n - i)\, x_{t-i}}{\frac{n(n+1)}{2}}
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Lookback is $n - 1$, as in TA-Lib.
- The weighted sum is advanced, not rebuilt: adding $n\,x_{t+1}$ and subtracting
  the plain window sum shifts every older weight down by one. Recomputing the
  sum each bar would make a batch run quadratic in the period.
- `period = 1` performs no smoothing and returns `source` unchanged.
- Leading `NaN` rows are skipped and the lookback counts from the first valid
  bar; a `NaN` or infinity after it raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

smoothed = tl.wma(close, period=30)
```

## References

- John J. Murphy, *Technical Analysis of the Financial Markets*, New York
  Institute of Finance, 1999, chapter 9.
