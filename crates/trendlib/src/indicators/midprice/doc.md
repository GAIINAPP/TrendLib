# Midpoint Price over Period

Halfway between the highest high and the lowest low of the last `period` bars.
Unlike `midpoint` it reads the bars' own extremes rather than a single series,
so an intraday spike counts even if the close recovered.

## Formula

$$
\mathrm{MIDPRICE}_t = \frac{\max_{i=t-n+1}^{t} h_i + \min_{i=t-n+1}^{t} l_i}{2}
$$

where $h$ and $l$ are `high` and `low`, and $n$ is `period`.

## Conventions

- Lookback is $n - 1$.
- Not recursive: each row depends only on the last $n$ bars.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

centre = tl.midprice(df, period=14)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
