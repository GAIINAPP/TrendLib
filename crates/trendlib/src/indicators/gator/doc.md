# Gator Oscillator

Bill Williams' Gator Oscillator: how far apart the Alligator's lines are, the
jaw-to-teeth distance above zero and the teeth-to-lips distance below it.

## Formula

With the jaw, teeth and lips of `alligator` at the same parameters, as drawn
at bar $t$:

$$
\text{gator\_upper}_t = |\text{jaw}_t - \text{teeth}_t| \qquad
\text{gator\_lower}_t = -|\text{teeth}_t - \text{lips}_t|
$$

## Conventions

- The distances are between the lines as drawn on this row, each already shifted
  ahead by its own `shift`, which is how Williams plots the histogram.
- Warm-up is the Alligator's: the largest of `period - 1 + shift` over the three
  lines.
- Recursive through the Alligator's smoothed averages.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `MEDPRICE`, `SMA` (the seed) and `SUB`, with the smoothing step in
  NumPy.

## Example

```python
import trendlib as tl
upper, lower = tl.gator(high, low)
```

## References

- Bill Williams, New Trading Dimensions, Wiley, 1998
