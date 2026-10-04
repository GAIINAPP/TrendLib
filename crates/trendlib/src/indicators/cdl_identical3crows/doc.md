# Identical Three Crows

Three black bars each opening where the last one closed and closing lower

## Formula

Present when three black bars close successively lower, each close near its own
low, and each bar opens at the previous close within the `equal` threshold.

## Conventions

- Warm-up is 12 bars.
- Each black bar is held to a lower shadow of at most a tenth of the recent
  average bar range, so the closes are near the lows.
- Unlike `cdl_3blackcrows` the opens are pinned to the previous closes rather
  than merely inside the previous bodies, and no preceding white bar is
  required.
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
found = tl.cdl_identical3crows(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
