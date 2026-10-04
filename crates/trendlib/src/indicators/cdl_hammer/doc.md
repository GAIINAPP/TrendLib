# Hammer

A short body with a long tail below it, sitting near the previous low

## Formula

Present when all four hold: the body is shorter than the recent average body,
the lower shadow is longer than the body itself, the upper shadow is at most a
tenth of the recent average range, and

$$
\min(O_t, C_t) \le L_{t-1} + 0.2 \cdot \frac{1}{5}\sum_{i=t-6}^{t-2} (H_i - L_i)
$$

so the body sits near the previous bar's low.

## Conventions

- Warm-up is 11 bars: ten for the body and shadow averages and one more for
  the previous bar they are read against.
- The `near` average is taken at the previous bar, not at this one.
- The colour of the body does not matter.
- `cdl_hangingman` asks the same four questions; the two differ only in where
  the bar sits in the trend, which neither of them checks.
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
found = tl.cdl_hammer(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
