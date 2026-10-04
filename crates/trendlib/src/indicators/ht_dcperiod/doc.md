# Hilbert Transform Dominant Cycle Period

How long the dominant cycle in the series currently is, in bars.

## Formula

The series is smoothed, detrended and split into a part in phase with the
dominant cycle and a part a quarter turn ahead of it. The cycle's length is a
full turn divided by how far the phase moved in a bar,

$$
\text{period}_t = \frac{360}{\arctan(\mathrm{Im}_t / \mathrm{Re}_t)}
$$

held within half of the previous length, clamped between 6 and 50, and then
smoothed twice.

## Conventions

- Warm-up is 32 bars. The transform itself starts twelve bars in, and the
  cycle length it reads has to settle before the first row is reported.
- The transform's buffers start empty, so the bars before it begins count as
  zero rather than as the values they had. That is TA-Lib's warm-up and it is
  what the first reported rows are built on.
- Recursive throughout, so the result is path dependent
  (`CONVENTIONS.md` § 5); `unstable` applies.
- The value is bounded between 6 and 50 by construction, so a series with no
  cycle in it still reads something.

## Example

```python
import trendlib as tl
ht_dcperiod = tl.ht_dcperiod(source)
```

## References

- John F. Ehlers, Rocket Science for Traders, Wiley, 2001
