# Tasuki Gap

A gap, then a bar of the other colour that opens inside the gapping body and closes into the gap without filling it

## Formula

Present when the second body gaps clear of the first, the third is the other
colour, opens inside the second body and closes inside the gap but short of the
first body, and the two bodies are of similar length within the `near`
threshold.

## Conventions

- Warm-up is 7 bars: five for the `near` average and two more for the bars it
  is read against.
- The gap is between the bodies, not the whole bars.
- The sign follows the gapping bar, since the gap is not filled.
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
found = tl.cdl_tasukigap(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
