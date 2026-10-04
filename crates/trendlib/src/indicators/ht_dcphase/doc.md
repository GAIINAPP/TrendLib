# Hilbert Transform Dominant Cycle Phase

Where the dominant cycle currently sits in its turn, in degrees.

## Formula

The smoothed series over one dominant cycle is correlated against a sine and a
cosine of that cycle, and the phase is the angle between the two sums:

$$
\text{phase}_t = \arctan\!\left(\frac{\sum \sin(\theta_i) s_{t-i}}{\sum \cos(\theta_i) s_{t-i}}\right)
\qquad \theta_i = \frac{2\pi i}{n_t}
$$

A quarter turn is added, then another `360 / period` degrees to make up the
weighted average's lag, and a half turn when the cosine sum is negative. The
result is wrapped to stay at or below 315 degrees.

## Conventions

- Warm-up is 63 bars. The transform starts thirty-seven bars in, twenty-five
  later than the readings that warm up in 32, and the cycle length it reads
  has to settle before the first row is reported.
- The transform's buffers start empty, so the bars before it begins count as
  zero rather than as the values they had.
- Recursive throughout, so the result is path dependent
  (`CONVENTIONS.md` § 5); `unstable` applies.
- When the cosine sum vanishes the reading is nudged a quarter turn rather than
  recomputed, so the previous bar's phase is part of this one's.
- The sums are over the **smoothed** series the transform works on, not the raw
  one.

## Example

```python
import trendlib as tl
ht_dcphase = tl.ht_dcphase(source)
```

## References

- John F. Ehlers, Rocket Science for Traders, Wiley, 2001
