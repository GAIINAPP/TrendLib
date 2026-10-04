# Hilbert Transform SineWave

The dominant cycle's phase drawn as a wave, with a second line an eighth of a turn ahead of it.

## Formula

$$
\mathrm{sine}_t = \sin(\text{phase}_t) \qquad
\mathrm{leadsine}_t = \sin(\text{phase}_t + 45^\circ)
$$

where the phase is `ht_dcphase`.

## Conventions

- Warm-up is 63 bars. The transform starts thirty-seven bars in, twenty-five
  later than the readings that warm up in 32, and the cycle length it reads
  has to settle before the first row is reported.
- The transform's buffers start empty, so the bars before it begins count as
  zero rather than as the values they had.
- Recursive throughout, so the result is path dependent
  (`CONVENTIONS.md` § 5); `unstable` applies.
- The two lines cross where the cycle turns, which is what `ht_trendmode`
  counts.
- Both columns start on the same row.

## Example

```python
import trendlib as tl
ht_sine_sine, ht_sine_lead_sine = tl.ht_sine(source)
```

## References

- John F. Ehlers, Rocket Science for Traders, Wiley, 2001
