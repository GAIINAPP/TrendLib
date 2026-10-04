# High Wave Candle

A short body with a very long wick on each side

## Formula

Present when the body is shorter than the recent average body and each shadow
is more than twice the body.

## Conventions

- Warm-up is 10 bars, the length of the body average.
- The very long shadow setting has no period of its own, so each shadow is
  measured against twice this bar's own body.
- The sign follows the bar's colour.
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
found = tl.cdl_highwave(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
