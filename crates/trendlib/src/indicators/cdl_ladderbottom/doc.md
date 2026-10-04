# Ladder Bottom

Three black bars stepping down, a fourth with a wick above, then a white one clearing it

## Formula

Present when three black bars open and close successively lower, a fourth black
bar grows an upper shadow longer than a tenth of the recent average bar range,
and a white bar then opens above its open and closes above its high.

## Conventions

- Warm-up is 14 bars.
- Only the fourth bar's shadow is measured against an average; the rest is the
  shape of the five bars.
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
found = tl.cdl_ladderbottom(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
