# Midprice

Halfway between the highest high and the lowest low of the last `period` bars:
the middle of the range the bars actually covered, rather than the middle of
their closes.

## Formula

$$
\mathrm{midprice}_t = \frac{\max_{i \in [t-n+1,\,t]} H_i + \min_{i \in [t-n+1,\,t]} L_i}{2}
$$

where $n$ is `period`.

## Conventions

- Warm-up is `period - 1` bars.
- The high and the low are tracked separately, so the two extremes need not
  come from the same bar. `midpoint` is the same idea taken from one series.
- Not path dependent.

## Example

```python
import trendlib as tl
middle = tl.midprice(high, low, period=14)
```

## References

- TA-Lib, `ta_MIDPRICE.c`.
