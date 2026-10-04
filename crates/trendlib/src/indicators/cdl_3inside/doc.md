# Three Inside Up/Down

A harami carried on by a third bar closing past the first bar's open

## Formula

Present when the first two bars form a harami, as in `cdl_harami`, and the third
closes beyond the first bar's open in the direction opposite to the first bar's
colour.

## Conventions

- Warm-up is 12 bars: ten for the body averages and two more for the bars they
  are read against.
- The sign is the opposite of the first bar's colour, which is what the harami
  already said; the third bar confirms it.
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
found = tl.cdl_3inside(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
