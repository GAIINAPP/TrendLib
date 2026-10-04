# Three-Line Strike

Three bars running one way, then one that opens past the last and closes past the first

## Formula

Present when three bars of one colour close successively further in the same
direction, each opening inside the previous body within the `near` threshold,
and a fourth bar of the other colour opens beyond the third's close and closes
beyond the first's open, undoing all three.

## Conventions

- Warm-up is 8 bars: five for the `near` average and three more for the bars it
  is read against.
- The sign follows the three bars that ran, not the one that struck them.
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
found = tl.cdl_3linestrike(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
