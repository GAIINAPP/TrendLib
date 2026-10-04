# Midpoint

Halfway between the highest and the lowest value of the last `period` bars. It
ignores where the series spent its time and reports only the middle of the
range it covered.

## Formula

$$
\mathrm{midpoint}_t = \frac{\max_{i \in [t-n+1,\,t]} x_i + \min_{i \in [t-n+1,\,t]} x_i}{2}
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Warm-up is `period - 1` bars.
- Both extremes come from the same series; `midprice` is the same idea taken
  from the high and the low separately.
- Not path dependent.

## Example

```python
import trendlib as tl
middle = tl.midpoint(close, period=14)
```

## References

- TA-Lib, `ta_MIDPOINT.c`.
