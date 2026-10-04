# Stick Sandwich

Two black bars closing at the same price with a white one between them

## Formula

Present when the first and third bars are black and close at the same price
within the `equal` threshold, and the white bar between them stays entirely
above the first bar's close.

## Conventions

- Warm-up is 7 bars: five for the `equal` average and two more for the bars it
  is read against.
- The `equal` average is taken at the first of the three, not at the last.
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
found = tl.cdl_sticksandwich(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
