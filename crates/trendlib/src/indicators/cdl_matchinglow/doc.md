# Matching Low

Two black bars that closed at the same price

## Formula

Present when both bars are black and

$$
|C_t - C_{t-1}| \le \text{equal}
$$

where `equal` is a twentieth of the average bar range over the five bars before
the first of the two.

## Conventions

- Warm-up is 6 bars: five for the `equal` average and one more for the bar it
  is read against.
- Nothing is asked of the bodies' lengths, only of where the two closed.
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
found = tl.cdl_matchinglow(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
