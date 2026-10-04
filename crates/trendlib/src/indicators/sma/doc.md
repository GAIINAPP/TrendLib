# Simple Moving Average

The unweighted mean of the last `period` values. It smooths a series by giving
every bar in the window the same weight, so it responds to a change only once
that change has entered the window, and it reflects the window's oldest bar as
strongly as its newest.

## Formula

$$
\mathrm{SMA}(x, n)_t = \frac{1}{n}\sum_{i=t-n+1}^{t} x_i
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Lookback is $n - 1$: the first $n - 1$ rows after the first valid bar are
  `NaN`, as in TA-Lib.
- The window sum is advanced rather than recomputed: add the newest value,
  divide, subtract the oldest. This reproduces TA-Lib's values bit for bit
  (`CONVENTIONS.md` section 1). A recomputed sum differs by a few ULP.
- `period = 1` performs no smoothing and returns `source` unchanged.
- Leading `NaN` rows are skipped and the lookback counts from the first valid
  bar; a `NaN` or infinity after it raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

smoothed = tl.sma(close, period=30)
```

## References

- John J. Murphy, *Technical Analysis of the Financial Markets*, New York
  Institute of Finance, 1999, chapter 9.
