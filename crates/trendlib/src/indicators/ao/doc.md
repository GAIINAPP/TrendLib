# Awesome Oscillator

The gap between a short and a long simple average of the bar midpoints.

## Formula

With $m_t = (H_t + L_t)/2$,

$$
\mathrm{ao}_t = \mathrm{SMA}(m, n_f)_t - \mathrm{SMA}(m, n_s)_t
$$

## Conventions

- Warm-up is the longer of the two periods, less one.
- The midpoint is used rather than the close, so where the bar finished does
  not matter, only how far it reached.
- Not path dependent.

## Example

```python
import trendlib as tl
ao = tl.ao(high, low)
```

## References

- Bill Williams, Trading Chaos, Wiley, 1995
