# Abandoned Baby

A doji stranded by a gap on both sides, between two long bars of opposite colours

## Formula

Present when a long bar is followed by a doji that gaps clear of it and then by
a long bar of the other colour that gaps clear of the doji the other way, closing
`penetration` of the way back into the first body.

## Conventions

- Warm-up is 12 bars.
- These gaps are between the whole bars, wicks included, not merely between the
  bodies, which is what makes the doji abandoned.
- Both outer bodies have to be long, and `penetration` defaults to 0.3.
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
found = tl.cdl_abandonedbaby(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
