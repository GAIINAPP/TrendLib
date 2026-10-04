# Fast Stochastic

Where the close sits inside the high-low range of the last few bars, read as a
percentage, together with a short average of that figure. A reading near 100
says the close finished at the top of its recent range and one near 0 at the
bottom.

## Formula

$$
\%K_t = 100 \cdot
\frac{C_t - \min_{i \in [t-n+1,\,t]} L_i}
     {\max_{i \in [t-n+1,\,t]} H_i - \min_{i \in [t-n+1,\,t]} L_i}
\qquad
\%D_t = \mathrm{MA}_{\text{type}}(\%K, m)_t
$$

where $n$ is `fastk_period`, $m$ is `fastd_period` and $\mathrm{MA}$ is
`fastd_ma_type`.

## Conventions

- A window whose high equals its low gives `0.0`. The test is on the exact
  range: a range that is merely small still places the close.
- Both columns start on the same row. `stochf_k` has a value `fastk_period - 1`
  bars in, but it is not reported until `stochf_d` has one, which is what
  TA-Lib does.
- Warm-up is `fastk_period - 1` plus the smoothing average's own warm-up.
- `fastd_ma_type` of `ema`, `rma`, `dema` or `tema` makes the result path
  dependent (`CONVENTIONS.md` § 5).

## Example

```python
import trendlib as tl
k, d = tl.stochf(high, low, close, fastk_period=5, fastd_period=3)
```

## References

- George C. Lane, "Lane's Stochastics", *Technical Analysis of Stocks and Commodities*, May-June 1984.
