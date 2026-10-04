# Belt Hold

A long body that opened at the end of its range and ran from there

## Formula

Present when the body is longer than the recent average body and the shadow on
the opening side is at most a tenth of the recent average bar range.

## Conventions

- Warm-up is 10 bars.
- This is `cdl_closingmarubozu` read from the other end: the opening side is
  the one held to a tenth of the range.
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
found = tl.cdl_belthold(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
