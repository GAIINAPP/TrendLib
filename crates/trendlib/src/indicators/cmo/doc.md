# Chande Momentum Oscillator

How one-sided recent movement has been, from -100 when every bar fell to +100
when every bar rose. It is the relative strength index's question asked
differently: the difference between gains and losses rather than the share of
the two that gains account for.

## Formula

$$
u_t = \max(x_t - x_{t-1},\, 0) \qquad d_t = \max(x_{t-1} - x_t,\, 0)
$$

$$
U_n = \sum_{i=1}^{n} u_i \qquad U_t = U_{t-1} - \frac{U_{t-1}}{n} + u_t,\quad t > n
$$

$$
\mathrm{CMO}_t = 100\,\frac{U_t - D_t}{U_t + D_t}
$$

with $D$ accumulated exactly as $U$, where $x$ is `source` and $n$ is `period`.

## Conventions

- Lookback is $n$, not $n - 1$: the first value needs $n$ bar-to-bar changes,
  which is $n + 1$ bars. This equals TA-Lib's lookback, which the tests assert.
- The gains and losses are accumulated as running totals rather than averages,
  seeded with the plain sum of the first $n$ and then decayed by $n$ each bar.
  Only the ratio is reported, so the scale cancels.
- When both totals are zero every bar in the window closed unchanged, and the
  value is reported as 0 rather than as a division by zero, as TA-Lib reports
  it.
- Recursive, so early rows still carry a trace of the seed (D9).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

swing = tl.cmo(close, period=14)
```

## References

- Tushar S. Chande and Stanley Kroll, *The New Technical Trader*, Wiley, 1994.
