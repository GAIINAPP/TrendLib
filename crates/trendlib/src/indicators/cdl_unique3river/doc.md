# Unique Three River

A long black bar, a smaller black one making a new low, then a short white one above it

## Formula

Present when a long black bar is followed by a shorter black bar whose body sits
inside the first's but whose low reaches below it, and then by a short white bar
opening above that low.

## Conventions

- Warm-up is 12 bars.
- Nothing is asked of the first bar's shadows. The description of the pattern
  calls for a long lower shadow there, but the oracle does not test it, and the
  one occurrence in the committed dataset does not have one.
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
found = tl.cdl_unique3river(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
