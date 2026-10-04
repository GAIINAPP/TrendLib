# Midpoint over Period

Halfway between the highest and the lowest value of the last `period` bars. It
ignores where the series spent its time and reports only the centre of the
range it covered.

## Formula

$$
\mathrm{MIDPOINT}_t = \frac{\max_{i=t-n+1}^{t} x_i + \min_{i=t-n+1}^{t} x_i}{2}
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Lookback is $n - 1$.
- Not recursive: each row depends only on the last $n$ bars.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

centre = tl.midpoint(close, period=14)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
