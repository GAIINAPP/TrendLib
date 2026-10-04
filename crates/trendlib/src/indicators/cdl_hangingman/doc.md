# Hanging Man

A short body with a long tail below it, sitting near the previous high

## Formula

The same four tests as `cdl_hammer`, with the last one reading from the other
end:

$$
\min(O_t, C_t) \ge H_{t-1} - 0.2 \cdot \frac{1}{5}\sum_{i=t-6}^{t-2} (H_i - L_i)
$$

so the body sits near the previous bar's high.

## Conventions

- Warm-up is 11 bars: ten for the body and shadow averages and one more for
  the previous bar the `near` average is read against.
- The output is `-100`: the same shape read at the top of a move rather than
  at the bottom.
- It is the **bottom** of the body that has to reach the previous high, not
  the top, so the whole body is up there. `cdl_hammer` is the mirror of that:
  the top of the body has to reach down to the previous low.
- Neither this nor `cdl_hammer` looks at where the bar sits in a trend, so
  both can fire anywhere.
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
found = tl.cdl_hangingman(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
