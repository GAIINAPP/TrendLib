# Harami Cross Pattern

A harami whose second bar is a doji rather than merely a short body

## Formula

The same as `cdl_harami` with the second body held to the doji threshold, a
tenth of the recent average bar range, instead of the average body length.

## Conventions

- Warm-up is 11 bars: ten for the body average and one more for the bar it is
  read against.
- The sign is the opposite of the previous bar's colour.
- Every harami cross is also a harami, since the doji threshold is the
  stricter of the two on all but the quietest stretches.
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
found = tl.cdl_haramicross(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
