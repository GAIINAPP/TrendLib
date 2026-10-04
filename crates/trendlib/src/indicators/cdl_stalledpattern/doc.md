# Stalled Pattern

Two long white bars then a short one riding on the second's shoulder

## Formula

Present when three white bars close successively higher, the first two bodies
are long, the second has almost no upper shadow, and the third body is short and
opens around the second's close.

## Conventions

- Warm-up is 12 bars.
- Only the **second** bar's upper shadow is tested. The first bar's is not,
  which is easy to assume and wrong: assuming it drops ten of the eleven
  occurrences in the committed dataset.
- The third bar's open is held within its own body length below the second's
  close, so a short body that gapped down does not qualify.
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
found = tl.cdl_stalledpattern(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
