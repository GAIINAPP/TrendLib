# Rising/Falling Three Methods

A long bar, three short ones drifting back inside its range, then a long one carrying on

## Formula

Present when a long bar is followed by three short bars of the other colour
that stay inside its high-low range and drift against it, and then by a long bar
of the first colour that opens beyond the third's close and closes beyond the
first's.

## Conventions

- Warm-up is 14 bars.
- The three middle bars are contained by the first bar's whole range, wicks
  included, not merely by its body.
- The direction is read from the first bar, so the same test covers the rising
  and the falling form.
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
found = tl.cdl_risefall3methods(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
