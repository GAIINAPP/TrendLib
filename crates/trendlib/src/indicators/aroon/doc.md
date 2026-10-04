# Aroon

How recently the highest high and the lowest low of the window were set, each on
a scale from 0 to 100. A reading of 100 means the extreme is this bar; 0 means
it is the oldest bar the window still holds. The pair says how fresh each
extreme is, not how far apart they are.

## Formula

$$
\mathrm{up}_t = 100\,\frac{n - \text{bars since the highest high}}{n} \qquad
\mathrm{down}_t = 100\,\frac{n - \text{bars since the lowest low}}{n}
$$

over the window $t-n \ldots t$, where $n$ is `period`.

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

down, up = tl.aroon(df, period=14)
```

## References

- Tushar S. Chande, Aroon, Technical Analysis of Stocks and Commodities, September 1995
