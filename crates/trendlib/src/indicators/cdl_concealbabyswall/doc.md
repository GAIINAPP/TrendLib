# Concealing Baby Swallow

Four black bars, two of them marubozu, then one that swallows the bar before it whole

## Formula

Present when four black bars run, the first two are marubozu, the third gaps
below the second but grows an upper shadow reaching back into its body, and the
fourth covers the third entirely, high and low.

## Conventions

- Warm-up is 13 bars.
- The third bar's upper shadow only has to exist, not to clear any average, but
  it does have to reach above the second bar's close.
- The gap is between the bodies, not the whole bars, which is what lets the
  shadow reach back.
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
found = tl.cdl_concealbabyswall(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
