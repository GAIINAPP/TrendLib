# Traditional Pivot Points

The floor-trader pivot and three levels either side of it.

## Formula

From the previous bar's high, low and close, with
$\mathrm{pp} = (H + L + C)/3$,

$$
\begin{aligned}
r_1 &= 2\,\mathrm{pp} - L & s_1 &= 2\,\mathrm{pp} - H\\
r_2 &= \mathrm{pp} + (H - L) & s_2 &= \mathrm{pp} - (H - L)\\
r_3 &= H + 2(\mathrm{pp} - L) & s_3 &= L - 2(H - \mathrm{pp})
\end{aligned}
$$

## Conventions

- Warm-up is 1 bar: row `t` describes the range in force during period `t`,
  built from the bar before it. Nothing is reported from the current bar.
- Inputs are bars of the period the levels are built from, normally daily
  bars. Deriving a prior day from intraday bars is M6.
- Not path dependent: each row depends on one earlier bar and nothing else.
- These are TradingView's "Pivot Points Standard" with type **Traditional**,
  levels 1 to 3.

## Example

```python
import trendlib as tl
pivots_traditional_pp, pivots_traditional_r1, pivots_traditional_r2, pivots_traditional_r3, pivots_traditional_s1, pivots_traditional_s2, pivots_traditional_s3 = tl.pivots_traditional(high, low, close)
```

## References

- Frank Ochoa, Secrets of a Pivot Boss, Wiley, 2010
