# Dark Cloud Cover

A black bar that opened above the previous high and closed well into the white body before it

## Formula

Present when the first body is white and longer than the recent average, the
second is black, it opens above the first's high, and

$$
O_{t-1} < C_t < C_{t-1} - p\,|C_{t-1} - O_{t-1}|
$$

where $p$ is `penetration`.

## Conventions

- Warm-up is 11 bars: ten for the body average and one more for the bar it is
  read against.
- `penetration` says how far into the previous body the close must reach, as a
  fraction of it. The oracle's default is 0.5, so a close at the midpoint is
  not enough.
- Only the first body has to be long; the second is held to where it closes.
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
found = tl.cdl_darkcloudcover(open, high, low, close)
```

## References

- Steve Nison, Japanese Candlestick Charting Techniques, New York Institute of Finance, 1991
