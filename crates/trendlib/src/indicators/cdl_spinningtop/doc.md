# Spinning Top

A short body with a wick longer than itself on each side

## Formula

Present when the body is shorter than the recent average body and each shadow
is longer than the body.

## Conventions

- Warm-up is 10 bars, the length of the body average.
- The shadows are measured against the body itself, not against an average, so
  only the body test looks back.
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
found = tl.cdl_spinningtop(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
