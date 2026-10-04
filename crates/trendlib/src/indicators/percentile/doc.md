# Percentile (nearest rank)

The value standing at a given percentile of the last `period` bars. At 50 it
is the window's median, at 0 its lowest value and at 100 its highest.

## Formula

Sort the window ascending and take

$$
\mathrm{percentile}_t = y_{\,\lceil n p / 100 \rceil}
$$

where $y_1 \le \dots \le y_n$ are the sorted values, $n$ is `period` and $p$ is
`percentile`. A rank of zero is read as one.

## Conventions

- Nearest rank, not interpolated: the answer is always one of the values in the
  window, never a point between two of them. At `period=5` and
  `percentile=33.3` that is the second smallest, where a linear percentile
  would return something between the second and the third.
- `percentile` is between 0 and 100 inclusive; the oracle refuses anything
  outside that and so does this.
- Warm-up is `period - 1` bars.
- Not path dependent.

## Example

```python
import trendlib as tl
median = tl.percentile(close, period=30, percentile=50.0)
```

## References

- TA-Lib, `ta_PERCENTILE.c`, nearest-rank percentile over a rolling window.
