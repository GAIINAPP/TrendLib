# On Neck Pattern

A white bar that opened under the previous low and closed back at it

## Formula

Present when the first body is black and long, the second is white, it opens
below the first's low and closes at that low, within the `equal` threshold.

## Conventions

- Warm-up is 11 bars: ten for the body average and one more, with the `equal`
  average over five taken at the previous bar.
- `equal` is a twentieth of the recent average bar range, so "the same price"
  means within that.
- The rally gives out exactly where the previous bar bottomed, which is why
  this reads bearish despite the white body.
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
found = tl.cdl_onneck(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
