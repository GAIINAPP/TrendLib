# Three Stars In The South

Three black bars, each smaller than the last and giving back less ground

## Formula

Present when a long black bar with a long tail is followed by a shorter black
bar that opens inside its range and dips less far, and then by a short black
marubozu entirely inside the second bar's range.

## Conventions

- Warm-up is 12 bars.
- The second bar must still reach below the first bar's close but not below its
  low, so the selling is losing ground rather than gaining it.
- The third bar is contained by the second, wicks included.
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
found = tl.cdl_3starsinsouth(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
