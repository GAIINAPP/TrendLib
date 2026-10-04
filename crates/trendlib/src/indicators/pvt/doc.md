# Price Volume Trend

A running total of each bar's volume weighted by how far the close moved, as a fraction.

## Formula

$$
\mathrm{pvt}_t = \mathrm{pvt}_{t-1} + \frac{C_t - C_{t-1}}{C_{t-1}}\,V_t
$$

## Conventions

- Lookback 0: the first bar starts the running total at zero.
- Path dependent (`CONVENTIONS.md` § 5): the value carries every bar before it,
  so a run started later begins its own total from zero.
- A previous close of exactly zero makes the step infinite rather than guarded;
  prices do not reach there.

## Example

```python
import trendlib as tl
pvt = tl.pvt(close, volume)
```

## References

- Joseph E. Granville, New Strategy of Daily Stock Market Timing for Maximum Profit, Prentice-Hall, 1976
