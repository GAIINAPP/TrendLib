# Homing Pigeon

A short black body inside the long black one before it

## Formula

Present when both bars are black, the first body is longer than the recent
average and the second shorter, and the second opens below the first's open and
closes above its close.

## Conventions

- Warm-up is 11 bars: ten for the body average and one more for the bar it is
  read against.
- This is a harami in which both bars are black, so the second body is inside
  the first without the colours having to differ.
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
found = tl.cdl_homingpigeon(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
