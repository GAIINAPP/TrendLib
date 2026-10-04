# Rolling Minimum

The lowest value seen in the last `period` bars. It moves only when a new
extreme arrives or when the bar holding the old one falls out of the window.

## Formula

$$
\mathrm{MIN}_t = \min_{i=t-n+1}^{t} x_i
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Lookback is $n - 1$.
- Ties keep the earlier bar, which matters only for `minindex`.
- Not recursive: each row depends only on the last $n$ bars.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

extreme = tl.min(close, period=30)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
