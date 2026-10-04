# Accelerator Oscillator

How fast the awesome oscillator is changing: the oscillator minus its own short average.

## Formula

$$
\mathrm{ac}_t = \mathrm{ao}_t - \mathrm{SMA}(\mathrm{ao}, n_g)_t
$$

## Conventions

- Warm-up is the awesome oscillator's plus the signal average's.
- This is a second difference, so it turns before the oscillator does and is
  noisier for it.
- Not path dependent.

## Example

```python
import trendlib as tl
ac = tl.ac(high, low)
```

## References

- Bill Williams, Trading Chaos, Wiley, 1995
