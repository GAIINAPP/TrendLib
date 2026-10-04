# Evening Doji Star

A long white bar, a doji one gapping up from it, then a black one closing well back into it

## Formula

Present when the first body is long and white, the second a doji and
gapping up clear of it, and the third is black and longer than the recent
average short body, with

$$
C_t < C_{t-2} - p\,|C_{t-2} - O_{t-2}|
$$

where $p$ is `penetration`.

## Conventions

- Warm-up is 12 bars.
- The gaps are between the bodies, not the whole bars.
- `penetration` says how far into the first body the third must close, as a
  fraction of it. The oracle's default is 0.3.
- The middle bar's colour is not tested; only its size.
- The middle bar is held to the doji threshold rather than to the average
  short body, which is the only difference from `cdl_eveningstar`.
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
found = tl.cdl_eveningdojistar(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
