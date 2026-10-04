# Hilbert Transform Phasor Components

The two parts the series is split into: one in phase with the dominant cycle, one a quarter turn ahead.

## Formula

The in-phase part is the detrended series three bars back; the quadrature part
is the four-tap Hilbert transform of it. Both are scaled by the current cycle
length before being reported.

## Conventions

- Warm-up is 32 bars. The transform itself starts twelve bars in, and the
  cycle length it reads has to settle before the first row is reported.
- The transform's buffers start empty, so the bars before it begins count as
  zero rather than as the values they had. That is TA-Lib's warm-up and it is
  what the first reported rows are built on.
- Recursive throughout, so the result is path dependent
  (`CONVENTIONS.md` § 5); `unstable` applies.
- Both columns start on the same row.
- The two are in the units of the detrended series, not of the input, so their
  size says little on its own.

## Example

```python
import trendlib as tl
ht_phasor_in_phase, ht_phasor_quadrature = tl.ht_phasor(source)
```

## References

- John F. Ehlers, Rocket Science for Traders, Wiley, 2001
