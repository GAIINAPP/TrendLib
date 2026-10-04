# Upside/Downside Gap Three Methods

Two bars of one colour with a gap between them, then a third that closes the gap

## Formula

Present when the first two bars share a colour and the second's body gaps clear
of the first's, then the third bar is the other colour, opens inside the
second's body and closes inside the first's, filling the gap.

## Conventions

- Warm-up is 2 bars: no average is involved, only the three bars themselves.
- The gap is between the bodies, not the whole bars, so wicks may overlap.
- The sign follows the two bars that ran, not the one that filled the gap.
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
found = tl.cdl_xsidegap3methods(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
