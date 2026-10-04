# Mat Hold

A long white bar, three small ones drifting back without undoing it, then another long white one

## Formula

Present when a long white bar is followed by a black body gapping above it and
two more small bars that fall back but stay above the first bar's body, and then
by a long white bar that opens above all three and closes above their highs.

## Conventions

- Warm-up is 14 bars.
- The gap is between the bodies, not the whole bars.
- `penetration` is accepted for compatibility with the oracle's signature but
  does not enter the test; TA-Lib's own implementation does not read it either.
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
found = tl.cdl_mathold(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
