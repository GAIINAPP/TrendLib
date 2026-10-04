# Efficiency Ratio

How much of the ground covered over the window turned into net progress, between 0 and 1.

## Formula

$$
\mathrm{er}_t = \frac{|x_t - x_{t-n}|}{\sum_{i=t-n+1}^{t} |x_i - x_{i-1}|}
$$

## Conventions

- Warm-up is `period` bars: the window of changes needs one bar more than it
  has changes.
- A window that did not move at all reads `1.0`, fully efficient, which is the
  reading `kama` takes of the same ratio. The test is exact.
- This is the ratio `kama` adapts on, taken on its own.
- Not path dependent.

## Example

```python
import trendlib as tl
er = tl.er(source)
```

## References

- Perry J. Kaufman, Smarter Trading, McGraw-Hill, 1995
