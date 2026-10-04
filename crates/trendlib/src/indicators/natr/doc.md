# Normalized Average True Range

The average true range expressed as a percentage of the current close. Dividing
by price is what makes it comparable: a 20-rupee range means something different
on a 200-rupee share than on a 2000-rupee one, and this removes that difference.

## Formula

$$
\mathrm{NATR}_t = 100\,\frac{\mathrm{ATR}_t}{c_t}
$$

where $\mathrm{ATR}$ is the average true range and $c$ is `close`.

## Conventions

- Lookback is $n$, the same as `atr`.
- `period = 1` leaves the true range unsmoothed but still normalises it, so the
  output is a percentage at every period. TA-Lib returns the raw true range
  there instead, in price units; this is Deviation 6 in `CONVENTIONS.md`
  section 9, and it is the one period at which the two disagree.
- A close of exactly zero leaves the value undefined, and the row is `NaN`
  rather than an infinity. Prices are not required to be positive in TrendLib
  (`CONVENTIONS.md` section 3), so this is reachable on a synthetic series.
- Wilder smoothing and warm-up behave exactly as in `atr`.

## Example

```python
import trendlib as tl

span = tl.natr(df, period=14)
```

## References

- John Forman, "Cleaning Up Your Volatility Measure", *Technical Analysis of
  Stocks and Commodities*, May 2006.
