# Kicking

Two marubozu of opposite colours with a gap between them

## Formula

Present when both bars are marubozu, they are different colours, and the whole
of the second bar lies on the far side of the first: above it after a black
one, below it after a white one. The gap is between the bars themselves, wicks
included, not merely between the bodies.

## Conventions

- Warm-up is 11 bars: ten for the body average and one more for the bar it is
  read against.
- The sign follows the second bar, which is the one that jumped.
- `cdl_kickingbylength` asks exactly the same questions and answers with the
  colour of the longer body instead.
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
found = tl.cdl_kicking(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
