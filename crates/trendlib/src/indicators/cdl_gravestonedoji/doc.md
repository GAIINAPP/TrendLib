# Gravestone Doji

A doji that opened and closed at the bottom of its range, with a long wick above

## Formula

Present when the body is a doji, the lower shadow is at most a tenth of the
recent average bar range and the upper shadow is more than that.

## Conventions

- Warm-up is 10 bars.
- This is `cdl_dragonflydoji` turned over: both shadows are measured against a
  tenth of the recent average bar range, one under and one over.
- A doji has no direction, so the output is `100` or `0`.
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
found = tl.cdl_gravestonedoji(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
