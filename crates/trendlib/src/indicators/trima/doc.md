# Triangular Moving Average

A moving average that counts the middle of its window most heavily and tapers to
almost nothing at both ends. It is smoother than a simple average of the same
length and turns later.

## Formula

$$
\mathrm{TRIMA}(x, n) = \mathrm{SMA}\bigl(\mathrm{SMA}(x, n_1), n_2\bigr)
$$

## Conventions

- Lookback is $n - 1$.
- The triangular weighting comes from averaging twice: with an odd period both
  stages use $(n+1)/2$ bars, and with an even one they use $n/2$ and $n/2 + 1$.
- Not recursive: each row depends only on the last $n$ bars.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

smoothed = tl.trima(close, period=30)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
