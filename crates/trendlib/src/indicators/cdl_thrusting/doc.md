# Thrusting Pattern

A white bar that opened under the previous low and closed inside the black body but short of its middle

## Formula

Present when the first body is black and long, the second is white, it opens
below the first's low, and

$$
C_{t-1} + \text{equal} < C_t \le C_{t-1} + \tfrac{1}{2}|C_{t-1} - O_{t-1}|
$$

## Conventions

- Warm-up is 11 bars: ten for the body average and one more, with the `equal`
  average over five taken at the previous bar.
- `equal` is a twentieth of the recent average bar range, so "the same price"
  means within that.
- A close past the midpoint would be `cdl_piercing` instead; this is the
  weaker version that stops short.
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
found = tl.cdl_thrusting(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
