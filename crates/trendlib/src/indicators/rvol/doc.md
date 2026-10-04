# Relative Volume

How this bar's volume compares with what has been usual lately, as a multiple.

## Formula

$$
\mathrm{rvol}_t = \frac{V_t}{\frac{1}{n}\sum_{i=t-n}^{t-1} V_i}
$$

## Conventions

- Warm-up is `period` bars, one more than the average covers: the current bar
  is not part of the average it is measured against.
- A window of entirely zero volume divides by zero and gives an infinity, which
  is left as it is.
- Not path dependent.

## Example

```python
import trendlib as tl
rvol = tl.rvol(volume)
```

## References

- TA-Lib, ta_RVOL.c
