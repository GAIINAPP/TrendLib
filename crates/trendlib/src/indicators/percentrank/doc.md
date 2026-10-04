# Percent Rank

What share of the previous `period` values the current one is above, read as a
percentage. It says where the bar sits in its own recent history without
reference to how far apart the values are.

## Formula

$$
\mathrm{percentrank}_t = \frac{100}{n}\,\#\{\,i \in [t-n,\,t-1] : x_i < x_t\,\}
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Warm-up is `period` bars: the current bar is ranked against the `period` bars
  before it and is not itself in the window.
- The comparison is strict, so a value equal to one in the window does not
  count. A flat stretch therefore reads 0, not 100.
- A value above everything in its history reads exactly 100.
- Not path dependent.

## Example

```python
import trendlib as tl
rank = tl.percentrank(close, period=100)
```

## References

- TA-Lib, `ta_PERCENTRANK.c`.
