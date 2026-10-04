# Takuri Line

A dragonfly doji whose tail is very long rather than merely long

## Formula

Present when the body is a doji, the upper shadow is at most a tenth of the
recent average bar range and the lower shadow is more than twice the body.

## Conventions

- Warm-up is 10 bars.
- This is `cdl_dragonflydoji` with the tail held to twice the body instead of
  once, so every takuri is also a dragonfly doji.
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
found = tl.cdl_takuri(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
