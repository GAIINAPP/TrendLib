# Chaikin Accumulation/Distribution Oscillator

The gap between a short and a long exponential average of the accumulation/distribution line.

## Formula

$$
\mathrm{adosc}_t = \mathrm{EMA}(\mathrm{ad}, n_f)_t - \mathrm{EMA}(\mathrm{ad}, n_s)_t
$$

## Conventions

- Warm-up is the longer period less one.
- Both averages are seeded with the first value of the A/D line, not with a
  mean of the first `period` values. The line is cumulative and starts at the
  first bar, so there is nothing earlier to average.
- The A/D line itself is the shipped `ad`, so the two can never drift apart.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5).

## Example

```python
import trendlib as tl
adosc = tl.adosc(high, low, close, volume)
```

## References

- Marc Chaikin, as described in Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
