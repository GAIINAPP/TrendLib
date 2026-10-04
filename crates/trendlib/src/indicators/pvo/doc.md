# Percentage Volume Oscillator

The gap between a short and a long average of volume, read as a percentage of the long one.

## Formula

$$
\mathrm{pvo}_t = 100 \cdot \frac{\mathrm{MA}(V, n_f)_t - \mathrm{MA}(V, n_s)_t}{\mathrm{MA}(V, n_s)_t}
$$

## Conventions

- This is `ppo` applied to volume rather than to price, and shares its
  conventions: the two periods are sorted first, and a slow average of exactly
  zero reads `0.0`.
- Warm-up is the longer period's own warm-up for the chosen average.
- The default average is `ema`, as TA-Lib's.

## Example

```python
import trendlib as tl
pvo = tl.pvo(volume)
```

## References

- Gerald Appel, Technical Analysis - Power Tools for Active Investors, FT Press, 2005
