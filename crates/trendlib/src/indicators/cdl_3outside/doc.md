# Three Outside Up/Down

An engulfing pattern carried on by a third bar closing further in the same direction

## Formula

Present when the second bar engulfs the first, as in `cdl_engulfing`, and the
third closes beyond the second's close in the same direction.

## Conventions

- Warm-up is 3 bars: no average is involved, only the three bars themselves.
- The sign follows the engulfing bar, which is the second of the three.
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
found = tl.cdl_3outside(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
