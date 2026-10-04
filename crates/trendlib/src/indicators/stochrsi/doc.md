# Stochastic RSI

The stochastic formula applied to the RSI rather than to price. The RSI already
moves between 0 and 100, so this reads where it sits inside its own recent
range and reaches the ends of that range far more often than the RSI itself
reaches 0 or 100.

## Formula

$$
\%K_t = 100 \cdot
\frac{\mathrm{RSI}_t - \min_{i \in [t-n+1,\,t]} \mathrm{RSI}_i}
     {\max_{i \in [t-n+1,\,t]} \mathrm{RSI}_i - \min_{i \in [t-n+1,\,t]} \mathrm{RSI}_i}
\qquad
\%D_t = \mathrm{MA}_{\text{type}}(\%K, m)_t
$$

where the RSI is taken over `period` bars, $n$ is `fastk_period` and $m$ is
`fastd_period`.

## Conventions

- A window of RSI values with no range gives `0.0`. The test is on the exact
  range.
- Both columns start on the same row: `stochrsi_k` is not reported until
  `stochrsi_d` has a value, which is what TA-Lib does.
- Warm-up is the RSI's own warm-up plus `fastk_period - 1` plus the smoothing
  average's.
- The RSI is Wilder's, so the result is path dependent
  (`CONVENTIONS.md` § 5) whatever `fastd_ma_type` is.

## Example

```python
import trendlib as tl
k, d = tl.stochrsi(close, period=14, fastk_period=5, fastd_period=3)
```

## References

- Tushar S. Chande and Stanley Kroll, *The New Technical Trader*, Wiley, 1994.
