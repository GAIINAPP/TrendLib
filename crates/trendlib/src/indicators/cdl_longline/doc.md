# Long Line Candle

A long body with both wicks short, so the bar moved and stayed moved

## Formula

Present when the body is longer than the recent average body and both
shadows are shorter than half the recent average shadow:

$$
|C_t - O_t| > \frac{1}{10}\sum_{i=t-10}^{t-1} |C_i - O_i|
$$

## Conventions

- Warm-up is 10 bars, the length of both averages.
- The shadow average is over both shadows added together, so the threshold each
  shadow is held to is half of it.
- The sign follows the bar's colour.
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
found = tl.cdl_longline(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
