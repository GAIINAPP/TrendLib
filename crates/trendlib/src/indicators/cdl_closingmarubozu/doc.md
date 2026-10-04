# Closing Marubozu

A long body that closed at the end of its range, whatever the other wick did

## Formula

Present when the body is longer than the recent average body and the shadow on
the closing side is at most a tenth of the recent average bar range. The shadow
on the opening side is not tested.

## Conventions

- Warm-up is 10 bars.
- Only the closing end is held to anything, which is what separates this from
  `cdl_marubozu`.
- The sign follows the bar's colour.
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
found = tl.cdl_closingmarubozu(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
