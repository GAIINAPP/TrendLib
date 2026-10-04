# Doji

A bar whose open and close are close enough together to count as the same price

## Formula

A doji is present when

$$
|C_t - O_t| \le 0.1 \cdot \frac{1}{10}\sum_{i=t-10}^{t-1} (H_i - L_i)
$$

that is, when the body is at most a tenth of the average bar range over the ten
bars before it.

## Conventions

- Warm-up is 10 bars, the length of the average the body is measured against.
- The comparison is against the ten bars **before** this one, so a bar is never
  measured against itself.
- Only the bullish form exists: a doji has no direction, so the output is `100`
  or `0`.
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
found = tl.cdl_doji(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
