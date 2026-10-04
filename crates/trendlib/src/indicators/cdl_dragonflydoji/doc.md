# Dragonfly Doji

A doji that opened and closed at the top of its range, with a long tail below

## Formula

Present when the body is a doji, the upper shadow is at most a tenth of the
recent average bar range and the lower shadow is more than that.

## Conventions

- Warm-up is 10 bars.
- Both shadows are measured against the same threshold, a tenth of the recent
  average bar range: one has to be under it and the other over it.
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
found = tl.cdl_dragonflydoji(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
