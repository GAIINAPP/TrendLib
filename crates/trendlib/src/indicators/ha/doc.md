# Heikin-Ashi Candles

A redrawn candle series in which each candle is built partly from the one
before it. The averaging smooths single-bar noise out of the body, so a run of
candles in one direction stays one colour where the raw bars alternate.

## Formula

$$
\mathrm{ha\_close}_t = \frac{O_t + H_t + L_t + C_t}{4} \qquad
\mathrm{ha\_open}_t = \frac{\mathrm{ha\_open}_{t-1} + \mathrm{ha\_close}_{t-1}}{2}
$$

$$
\mathrm{ha\_high}_t = \max(H_t,\ \mathrm{ha\_open}_t,\ \mathrm{ha\_close}_t) \qquad
\mathrm{ha\_low}_t = \min(L_t,\ \mathrm{ha\_open}_t,\ \mathrm{ha\_close}_t)
$$

The first candle has no predecessor and opens at $(O_0 + C_0)/2$.

## Conventions

- Lookback 0: every bar has a candle, including the first.
- `ha_open` carries every candle before it, so the result is path dependent
  (`CONVENTIONS.md` § 5): a slice of the data opens its first candle from the
  bar itself rather than from the candle that would have preceded it.
- The candles are not prices. A Heikin-Ashi close is an average of four
  numbers and no trade happened at it.

## Example

```python
import trendlib as tl
ha_open, ha_high, ha_low, ha_close = tl.ha(open, high, low, close)
```

## References

- Dan Valcu, "Heikin-Ashi How-to", *Technical Analysis of Stocks and Commodities*, February 2004.
