# Two Crows

A long white bar, a black one gapping above it, then a black one closing back inside it

## Formula

Present when the first bar is white and longer than the recent average, the
second is black and its body gaps above the first's, and the third is black,
opens inside the second's body and closes inside the first's.

## Conventions

- Warm-up is 12 bars.
- The gap is between the bodies, not the whole bars.
- The third bar must close above the first bar's open but below its close, so
  it eats into the white body without undoing it.
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
found = tl.cdl_2crows(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
