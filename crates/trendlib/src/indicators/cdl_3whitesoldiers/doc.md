# Three Advancing White Soldiers

Three white bars each opening inside the last body and closing higher without shortening

## Formula

Present when three white bars close successively higher, each close near its own
high, each opening above the previous open but not far above the previous close,
and no body much shorter than the one before it.

## Conventions

- Warm-up is 12 bars.
- `near` bounds how far above the previous close a bar may open and `far` how
  much shorter its body may be, so the advance has to be steady rather than a
  jump followed by a stall.
- The last body must still be longer than the recent average short body.
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
found = tl.cdl_3whitesoldiers(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
