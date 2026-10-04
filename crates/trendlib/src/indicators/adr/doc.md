# Average Day Range

The plain average of the last few bars' high-low ranges.

## Formula

$$
\mathrm{adr}_t = \frac{1}{n}\sum_{i=t-n+1}^{t} (H_i - L_i)
$$

## Conventions

- Warm-up is `period - 1` bars.
- Unlike `atr` this ignores gaps: a bar that opened far from the last close
  still counts only its own range.
- Not path dependent.

## Example

```python
import trendlib as tl
adr = tl.adr(high, low)
```

## References

- TA-Lib, ta_ADR.c
