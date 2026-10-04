# Piercing Pattern

A long white bar that opened under the previous low and closed past the middle of the black body before it

## Formula

Present when both bodies are longer than the recent average, the first is black
and the second white, the second opens below the first's low, and

$$
C_{t-1} + \tfrac{1}{2}|C_{t-1} - O_{t-1}| < C_t < O_{t-1}
$$

## Conventions

- Warm-up is 11 bars: ten for the body average and one more for the bar it is
  read against.
- Both bodies have to be long, not just the first.
- The close must clear the midpoint of the previous body but stay under its
  open; a close past the open would be an engulfing instead.
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
found = tl.cdl_piercing(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
