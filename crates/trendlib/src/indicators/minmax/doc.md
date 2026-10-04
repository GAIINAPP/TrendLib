# Rolling Minimum and Maximum

The lowest and the highest value seen in the last `period` bars, reported
together. The pair describes the band the series has stayed inside.

## Formula

$$
\mathrm{min}_t = \min_{i=t-n+1}^{t} x_i \qquad
\mathrm{max}_t = \max_{i=t-n+1}^{t} x_i
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Lookback is $n - 1$; both outputs share it.
- Not recursive: each row depends only on the last $n$ bars.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

low, high = tl.minmax(close, period=30)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
