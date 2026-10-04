# Upside Gap Two Crows

A long white bar, then two black ones above it, the second swallowing the first without closing the gap

## Formula

Present when the first bar is white and longer than the recent average, the
second is black with its body gapping above it, and the third is black, engulfs
the second's body and still closes above the first's.

## Conventions

- Warm-up is 12 bars.
- The gaps are between the bodies, not the whole bars.
- `cdl_2crows` is the version where the third bar closes back inside the white
  body instead of staying above it.
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
found = tl.cdl_upsidegap2crows(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
