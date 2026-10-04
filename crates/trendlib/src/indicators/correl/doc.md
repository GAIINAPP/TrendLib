# Pearson Correlation

How closely two series moved together over the window, between -1 and 1.

## Formula

$$
\mathrm{correl}_t = \frac{\sum xy - \frac{1}{n}\sum x \sum y}
{\sqrt{\left(\sum x^2 - \frac{1}{n}(\sum x)^2\right)\left(\sum y^2 - \frac{1}{n}(\sum y)^2\right)}}
$$

over the `period` bars ending at $t$.

## Conventions

- Warm-up is `period - 1` bars.
- A window in which either series never moves reads `0.0`: there is no spread
  for the other to be correlated with. The test is on the exact product of the
  two variances.
- All five sums are re-added from the window each bar rather than carried
  forward, because the numerator subtracts two quantities of the same size.
- Squaring a value overflows above about `1e154`, so a finite series of that
  magnitude gives a non-finite correlation.
- Not path dependent.

## Example

```python
import trendlib as tl
correl = tl.correl(source0, source1)
```

## References

- TA-Lib, ta_CORREL.c
