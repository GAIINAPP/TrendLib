# Separating Lines

A long bar that opened where the opposite-coloured bar before it opened, and ran the other way

## Formula

Present when the two bars are different colours, they open at the same price
within the `equal` threshold, the second body is longer than the recent average
and the shadow on its opening side is at most a tenth of the recent average bar
range.

## Conventions

- Warm-up is 11 bars: ten for the body average and one more, with the `equal` average over five taken at the previous bar.
- `equal` is a twentieth of the recent average bar range, so "the same price"
  means within that.
- The sign follows the second bar, which is the one that moved.
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
found = tl.cdl_separatinglines(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
