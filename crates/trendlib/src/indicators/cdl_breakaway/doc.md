# Breakaway

A gap away from a long bar, three bars drifting further, then one closing back into the gap

## Formula

Present when a long bar is followed by a body gapping clear of it and three more
bars drifting the same way, and then by a bar of the other colour that closes
back inside the gap: beyond the gapping bar's open but short of the long bar's
close.

## Conventions

- Warm-up is 14 bars.
- The third bar's colour is not tested; the first, second, fourth and fifth are.
- The two middle bars have to keep making ground in the same direction, high
  and low together.
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
found = tl.cdl_breakaway(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
