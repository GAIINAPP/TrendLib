# Harami Pattern

A short body sitting entirely inside the long body before it

## Formula

Present when the previous body is longer than the recent average, this body is
shorter than it, and this body lies strictly inside the previous one:

$$
\max(O_t, C_t) < \max(O_{t-1}, C_{t-1}) \quad\text{and}\quad
\min(O_t, C_t) > \min(O_{t-1}, C_{t-1})
$$

## Conventions

- Warm-up is 11 bars: ten for the body average and one more for the bar it is
  read against.
- The sign is the **opposite** of the previous bar's colour: a short body
  inside a long black one is read as bullish.
- Only the bodies are compared; the wicks may reach past the previous bar.
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
found = tl.cdl_harami(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
