# Moving Average Convergence Divergence

The gap between a faster and a slower exponential average of the same series,
together with an exponential average of that gap and the difference between the
two. It measures how far the short-run average has pulled away from the
long-run one, which widens as a move accelerates and narrows as it stalls.

## Formula

$$
\mathrm{macd}_t = \mathrm{EMA}(x, n_f)_t - \mathrm{EMA}(x, n_s)_t
$$

$$
\mathrm{signal}_t = \mathrm{EMA}(\mathrm{macd}, n_g)_t \qquad
\mathrm{hist}_t = \mathrm{macd}_t - \mathrm{signal}_t
$$

where $x$ is `source`, $n_f$ is `fast_period`, $n_s$ is `slow_period` and
$n_g$ is `signal_period`.

## Conventions

- Lookback is $\max(n_f, n_s) - 1 + n_g - 1$. All three outputs share it: the
  difference is not reported before its signal line exists, even though the two
  averages are defined a little earlier.
- **The shorter average is started late**, by exactly the difference in
  periods, so that both averages reach their first value on the same bar. This
  matters and is easy to get wrong: running both from the first bar instead
  would give the faster average a longer history at every point and shift the
  whole line. TA-Lib does the same, which is why the two agree.
- Seeding and smoothing follow `ema`; the result is recursive and there is no
  unstable-period setting (D9).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

line, signal, hist = tl.macd(close, fast_period=12, slow_period=26, signal_period=9)
```

## References

- Gerald Appel, *Technical Analysis: Power Tools for Active Investors*, FT
  Press, 2005.
