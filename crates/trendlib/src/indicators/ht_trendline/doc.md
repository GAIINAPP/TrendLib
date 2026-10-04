# Hilbert Transform Instantaneous Trendline

The series averaged over exactly one dominant cycle, then smoothed, which takes the cycle out and leaves the trend.

## Formula

$$
\bar{x}_t = \frac{1}{n_t}\sum_{i=0}^{n_t-1} x_{t-i}
\qquad n_t = \lfloor \text{period}_t + 0.5 \rfloor
$$

$$
\mathrm{trendline}_t = \frac{4\bar{x}_t + 3\bar{x}_{t-1} + 2\bar{x}_{t-2} + \bar{x}_{t-3}}{10}
$$

## Conventions

- Warm-up is 63 bars. The transform starts thirty-seven bars in, twenty-five
  later than the readings that warm up in 32, and the cycle length it reads
  has to settle before the first row is reported.
- The transform's buffers start empty, so the bars before it begins count as
  zero rather than as the values they had.
- Recursive throughout, so the result is path dependent
  (`CONVENTIONS.md` § 5); `unstable` applies.
- The average is over the **raw** series, not the smoothed one the transform
  works on. The dominant cycle is held to fifty bars, so the window never
  reaches further.

## Example

```python
import trendlib as tl
ht_trendline = tl.ht_trendline(source)
```

## References

- John F. Ehlers, Rocket Science for Traders, Wiley, 2001
