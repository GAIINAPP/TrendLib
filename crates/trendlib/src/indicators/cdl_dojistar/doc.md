# Doji Star

A doji whose body gaps clear of the long body before it

## Formula

Present when the previous body is longer than the recent average, this one is a
doji, and the doji's body gaps away from it in the direction the previous bar
was going.

## Conventions

- Warm-up is 11 bars.
- The gap is between the bodies, not the whole bars.
- The sign is the opposite of the previous bar's colour: the move stalled.
- This is the two-bar start of `cdl_morningdojistar` and
  `cdl_eveningdojistar`, which wait for a third bar to confirm it.
- The output is `100` where the pattern is present, `-100` where its bearish
  form is and `0` otherwise. An `int32` column warms up with `0`, not `NaN`
  (`CONVENTIONS.md` § 2).
- The candle settings are TA-Lib's defaults and are fixed in 0.1; choosing
  them is M6.
- Not path dependent: the answer depends on the bars in the window and
  nothing before them.

## Example

```python
import trendlib as tl
found = tl.cdl_dojistar(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
