# Tristar Pattern

Three doji in a row, the middle one gapping clear and the third coming back

## Formula

Present when all three bodies are doji, the second gaps clear of the first and
the third's body falls back towards it.

## Conventions

- Warm-up is 12 bars.
- All three bodies are measured against the **same** average, the one trailing
  the first of the three. TA-Lib keeps one running total here where its other
  three-bar patterns keep one per position, and the difference is visible: the
  per-position reading misses the one occurrence in the committed dataset.
- The gaps are between the bodies, not the whole bars.
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
found = tl.cdl_tristar(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
