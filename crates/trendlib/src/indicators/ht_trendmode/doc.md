# Hilbert Transform Trend vs Cycle Mode

Whether the series is currently trending or cycling, as one or zero.

## Formula

The reading is a trend unless one of three things says otherwise: the sine and
lead-sine lines have just crossed, fewer than half a cycle has passed since they
last did, or the phase is advancing at about one cycle's worth per bar. Any of
those makes it a cycle. A series sitting more than 1.5 per cent away from its
own `ht_trendline` is called a trend whatever the rest said.

## Conventions

- Warm-up is 63 bars, as for the rest of the readings that start thirty-seven
  bars in.
- The output is `1` or `0`; an `int32` column warms up with `0`, not `NaN`
  (`CONVENTIONS.md` § 2), so a warm-up row and a cycling row read the same.
- The distance test is against the **smoothed** series, while the trendline it
  is compared with is built from the raw one.
- Recursive throughout, so the result is path dependent
  (`CONVENTIONS.md` § 5); `unstable` applies.

## Example

```python
import trendlib as tl
ht_trendmode = tl.ht_trendmode(source)
```

## References

- John F. Ehlers, Rocket Science for Traders, Wiley, 2001
