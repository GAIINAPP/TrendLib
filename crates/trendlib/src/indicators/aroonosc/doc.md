# Aroon Oscillator

The difference between the two Aroon readings, from -100 to 100. It is positive
when the highest high is the fresher of the two extremes and negative when the
lowest low is, and says nothing about how large either move was.

## Formula

$$
\mathrm{AROONOSC}_t = \mathrm{up}_t - \mathrm{down}_t
$$

where $\mathrm{up}$ and $\mathrm{down}$ are the two Aroon readings over the
window $t-n \ldots t$.

## Conventions

- Lookback is $n$: the window holds $n + 1$ bars, the current one and the $n$
  before it.
- An extreme set on the current bar reads 100 and one about to leave the window
  reads 0.
- Ties keep the most recent bar, so a flat stretch reads 100 rather than
  decaying towards 0.
- Not recursive: each row depends only on the last $n + 1$ bars.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

swing = tl.aroonosc(df, period=14)
```

## References

- Tushar S. Chande, Aroon, Technical Analysis of Stocks and Commodities, September 1995
