# Up/Down-gap Side-by-side White Lines

Two white bars of the same size opening at the same price, both on the far side of a gap

## Formula

Present when the second and third bodies both gap clear of the first in the same
direction, both are white, their bodies are of similar length within the `near`
threshold and they open at the same price within the `equal` threshold.

## Conventions

- Warm-up is 7 bars.
- The sign follows the direction of the gap, not the colour of the bars: two
  white bars below a gap down read bearish.
- The gaps are between the bodies, not the whole bars.
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
found = tl.cdl_gapsidesidewhite(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
