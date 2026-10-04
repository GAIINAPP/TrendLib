# Shooting Star

A short body with a long wick above it, opening above the previous body

## Formula

The same three bar tests as `cdl_invertedhammer`, with the body gapping above
the previous one's rather than below it.

## Conventions

- Warm-up is 11 bars.
- The gap is between the bodies, not the whole bars.
- The output is `-100`: the long wick above says the push up was given back.
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
found = tl.cdl_shootingstar(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
