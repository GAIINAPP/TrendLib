# Advance Block

Three white bars closing higher, with the advance visibly slowing

## Formula

Present when three white bars close successively higher, each opening inside the
previous body, the first body is long with a short upper shadow, and the advance
is weakening in one of four ways: the bodies shortening past the `far`
threshold, or shortening at all while an upper shadow grows long.

## Conventions

- Warm-up is 12 bars.
- The four ways of weakening are alternatives, not all required; TA-Lib takes
  whichever applies.
- `cdl_3whitesoldiers` is the same three bars with the weakening tests
  inverted, so the two are mutually exclusive by construction.
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
found = tl.cdl_advanceblock(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
