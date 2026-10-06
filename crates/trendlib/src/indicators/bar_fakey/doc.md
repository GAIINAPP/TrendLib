# Fakey

Marks a close beyond the bar an inside bar sat within: above its high (+100) or
below its low (-100), the bar after the inside bar.

## Formula

With $I_t = H_{t-1} < H_{t-2} \wedge L_{t-1} > L_{t-2}$:

$$
\text{bar\_fakey}_t = 100\,[I_t \wedge C_t > H_{t-2}] - 100\,[I_t \wedge C_t < L_{t-2}]
$$

## Conventions

- Definitions and defaults are those of `ta-patterns` 1.2.1's `fakey` (oracle P,
  `INDICATORS.md` § 5.2).
- The two directions are the oracle's two halves combined its own way: each adds
  its sign and the sum is clipped, so a bar that answers both reads `0`.
- This is ta-patterns' reading of the fakey, its `hikkake_bullish` and
  `hikkake_bearish`; it is not TA-Lib's `CDLHIKKAKE`, which `cdl_hikkake`
  follows and which also scores a later confirmation.
- Warm-up is 2 bars, the first bar the oracle reads. An `int32` column warms up
  with `0`, not `NaN` (`CONVENTIONS.md` § 2), and the output says what price did
  on these bars; it names no price level.
- Not path dependent: the answer depends on the bars in the window and nothing
  before them.

## Example

```python
import trendlib as tl
found = tl.bar_fakey(open, high, low, close)
```

## References

- Daniel L. Chesler, Trading False Moves with the Hikkake Pattern, Active Trader, 2004
- ta-patterns 1.2.1, ta_patterns.chart_patterns.fakey (oracle P)
