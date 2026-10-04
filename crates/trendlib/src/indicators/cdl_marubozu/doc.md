# Marubozu

A long body with almost no wick at either end

## Formula

Present when the body is longer than the recent average body and both
shadows are at most a tenth of the recent average bar range.

## Conventions

- Warm-up is 10 bars.
- A marubozu is a long line with the wicks held to a tenth of the range rather
  than to the average shadow, so it is the stricter of the two.
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
found = tl.cdl_marubozu(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
