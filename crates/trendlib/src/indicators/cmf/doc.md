# Chaikin Money Flow

What share of the volume over the last few bars traded in the upper half of its bar.

## Formula

With $f_t = \dfrac{(C_t - L_t) - (H_t - C_t)}{H_t - L_t}V_t$,

$$
\mathrm{cmf}_t = \frac{\sum_{i=t-n+1}^{t} f_i}{\sum_{i=t-n+1}^{t} V_i}
$$

## Conventions

- Warm-up is `period - 1` bars.
- A bar whose high equals its low contributes zero flow rather than dividing by
  zero; its volume still counts in the denominator.
- A window in which nothing traded gives `0.0`, as TA-Lib does. The test is on
  the exact total.
- Both sums are re-added from the window each bar rather than carried forward.
- Not path dependent.

## Example

```python
import trendlib as tl
cmf = tl.cmf(high, low, close, volume)
```

## References

- Marc Chaikin, as described in Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
