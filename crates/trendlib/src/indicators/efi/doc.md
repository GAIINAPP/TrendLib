# Elder Force Index

How far the close moved times the volume that moved it, smoothed exponentially.

## Formula

$$
\mathrm{efi}_t = \mathrm{EMA}\big((C_t - C_{t-1})\,V_t,\ n\big)
$$

## Conventions

- Warm-up is `period` bars: one for the move and `period - 1` for the average.
- The product is a price times a volume, so the scale depends on both and the
  value is not comparable across instruments. It also overflows sooner than
  either does alone: above about `1e154` on both, the force is infinite.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5);
  `unstable` applies.

## Example

```python
import trendlib as tl
efi = tl.efi(close, volume)
```

## References

- Alexander Elder, Trading for a Living, Wiley, 1993
