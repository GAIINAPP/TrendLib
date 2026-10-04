# Inverted Hammer

A short body with a long wick above it, opening below the previous body

## Formula

Present when the body is shorter than the recent average, the upper shadow is
longer than the body, the lower shadow is at most a tenth of the recent average
bar range, and the whole body sits below the previous one's.

## Conventions

- Warm-up is 11 bars.
- The gap is between the bodies, not the whole bars, so the wicks may overlap.
- `cdl_shootingstar` asks the same questions with the gap the other way.
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
found = tl.cdl_invertedhammer(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
