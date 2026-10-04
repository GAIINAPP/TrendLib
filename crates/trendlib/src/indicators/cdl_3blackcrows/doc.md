# Three Black Crows

Three black bars each opening inside the last body and closing lower, after a white one

## Formula

Present when a white bar is followed by three black ones, each opening inside
the previous body and closing below the previous close, with each close near
its own low.

## Conventions

- Warm-up is 13 bars.
- Each black bar is held to a lower shadow of at most a tenth of the recent
  average bar range, so the closes are near the lows.
- Each open must be inside the previous body, not merely below the previous
  open, so a run of gaps down does not qualify.
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
found = tl.cdl_3blackcrows(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
