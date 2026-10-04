# Counterattack

Two long bars of opposite colours that closed at the same price

## Formula

Present when the two bars are different colours, both bodies are longer than
the recent average, and the two closes are within the `equal` threshold of each
other.

## Conventions

- Warm-up is 11 bars: ten for the body average and one more, with the `equal` average over five taken at the previous bar.
- `equal` is a twentieth of the recent average bar range, so "the same price"
  means within that.
- The sign follows the second bar.
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
found = tl.cdl_counterattack(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
