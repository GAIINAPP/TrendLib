# Engulfing Pattern

A body that covers the whole of the previous one, in the other colour

## Formula

Bullish when the bar is white, the one before it black, and

$$
C_t > O_{t-1} \quad\text{and}\quad O_t < C_{t-1}
$$

Bearish with the colours reversed and the comparisons swapped. The test is on
the bodies alone; the wicks are not part of it.

## Conventions

- Warm-up is 2 bars, which is what the oracle reserves even though only one
  bar of history is read.
- A bar that opened and closed at the same price counts as white, which is
  TA-Lib's reading of colour.
- Equal opens or closes do not engulf: both comparisons are strict.
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
found = tl.cdl_engulfing(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
