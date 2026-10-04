# Kicking by Length

A kicking pattern answered with the colour of the longer of its two bodies

## Formula

The same two tests as `cdl_kicking`. Where they are met, the answer follows the
colour of whichever body is longer rather than of the second bar.

## Conventions

- Warm-up is 11 bars: ten for the body average and one more for the bar it is
  read against.
- The two functions fire on exactly the same bars and differ only in sign,
  and then only where the first body is the longer of the two.
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
found = tl.cdl_kickingbylength(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
