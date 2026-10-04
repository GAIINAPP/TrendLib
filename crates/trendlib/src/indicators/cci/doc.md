# Commodity Channel Index

How far the typical price has strayed from its own recent average, measured in
units of how much it usually strays. Dividing by the mean deviation is what
makes the number comparable across instruments and across quiet and busy
periods.

## Formula

$$
\mathrm{TP}_t = \frac{h_t + l_t + c_t}{3} \qquad
\bar{\mathrm{TP}}_t = \frac{1}{n}\sum_{i=t-n+1}^{t} \mathrm{TP}_i
$$

$$
D_t = \frac{1}{n}\sum_{i=t-n+1}^{t}\bigl|\mathrm{TP}_i - \bar{\mathrm{TP}}_t\bigr|
\qquad
\mathrm{CCI}_t = \frac{\mathrm{TP}_t - \bar{\mathrm{TP}}_t}{0.015\,D_t}
$$

where $n$ is `period` and $0.015$ is Lambert's scaling constant.

## Conventions

- Lookback is $n - 1$.
- A window whose typical price never moved has no deviation to measure against,
  and the value is reported as 0 rather than as a division by zero, as TA-Lib
  reports it.
- The average is re-summed from the window on every bar rather than carried as
  a running total. The next step subtracts it from a number of the same size,
  which cancels most of the significant digits, so a running total's drift
  would show up in the result. This costs one pass over the window, which is
  what TA-Lib spends too.
- The index is ill-conditioned when the window's values lie within a unit or
  two in the last place of each other. The subtraction leaves no significant
  digits, and the mean must round to one of the values; which one it lands on
  decides the answer. TA-Lib rounds it differently and reports 0 where
  TrendLib reports a full-scale reading. Real price data, which moves in ticks,
  never reaches that state.
- Not recursive: each row depends only on the last $n$ bars.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

index = tl.cci(df, period=14)
```

## References

- Donald R. Lambert, "Commodity Channel Index: Tool for Trading Cyclic Trends",
  *Commodities*, October 1980.
