# Long Legged Doji

A doji with a long wick on at least one side

## Formula

Present when the body is a doji and at least one shadow is longer than the
body.

## Conventions

- Warm-up is 10 bars.
- Either shadow will do; `cdl_rickshawman` is the stricter form that wants both
  and the body in the middle.
- A doji has no direction, so the output is `100` or `0`.
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
found = tl.cdl_longleggeddoji(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
