# Rickshaw Man

A doji with a long wick on each side and its body near the middle of the range

## Formula

Present when the body is a doji, both shadows are longer than the body, and the
body straddles the middle of the bar's range within the `near` threshold:

$$
\min(O_t, C_t) \le L_t + \tfrac{1}{2}(H_t - L_t) + \text{near}
\quad\text{and}\quad
\max(O_t, C_t) \ge L_t + \tfrac{1}{2}(H_t - L_t) - \text{near}
$$

## Conventions

- Warm-up is 10 bars, the length of the doji average.
- The `near` average is taken at this bar, unlike in the hammer family where
  it is read at the previous one.
- This is the stricter form of `cdl_longleggeddoji`, which asks for only one
  long shadow and nothing about where the body sits.
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
found = tl.cdl_rickshawman(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
