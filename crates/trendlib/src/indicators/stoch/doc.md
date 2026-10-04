# Slow Stochastic

The fast stochastic with one more layer of smoothing. Where the close sits in
the recent high-low range is averaged once to give %K and again to give %D, so
the pair moves more slowly than `stochf` and crosses less often.

## Formula

$$
\text{raw}_t = 100 \cdot
\frac{C_t - \min_{i \in [t-n+1,\,t]} L_i}
     {\max_{i \in [t-n+1,\,t]} H_i - \min_{i \in [t-n+1,\,t]} L_i}
$$

$$
\%K_t = \mathrm{MA}_{\text{slowk}}(\text{raw}, p)_t \qquad
\%D_t = \mathrm{MA}_{\text{slowd}}(\%K, q)_t
$$

where $n$ is `fastk_period`, $p$ is `slowk_period` and $q$ is `slowd_period`.

## Conventions

- A window whose high equals its low gives `0.0` for the raw value. The test is
  on the exact range.
- Both columns start on the same row: `stoch_k` is not reported until `stoch_d`
  has a value, which is what TA-Lib does.
- Warm-up is `fastk_period - 1` plus both averages' own warm-ups.
- The two averages are chosen separately, so `slowk_ma_type` and
  `slowd_ma_type` need not match.
- An average of `ema`, `rma`, `dema` or `tema` makes the result path dependent
  (`CONVENTIONS.md` § 5).

## Example

```python
import trendlib as tl
k, d = tl.stoch(high, low, close, fastk_period=5, slowk_period=3, slowd_period=3)
```

## References

- George C. Lane, "Lane's Stochastics", *Technical Analysis of Stocks and Commodities*, May-June 1984.
