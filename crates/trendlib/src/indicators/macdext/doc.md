# MACD with Selectable Averages

MACD with the kind of each average left open. The line is still the shorter
average minus the longer one and the signal is still an average of that line,
but any of the three can be something other than exponential.

## Formula

$$
\mathrm{macd}_t = \mathrm{MA}_{\text{fast}}(x, n_f)_t - \mathrm{MA}_{\text{slow}}(x, n_s)_t
$$

$$
\mathrm{signal}_t = \mathrm{MA}_{\text{signal}}(\mathrm{macd}, n_g)_t \qquad
\mathrm{hist}_t = \mathrm{macd}_t - \mathrm{signal}_t
$$

where $n_f$, $n_s$ and $n_g$ are `fast_period`, `slow_period` and
`signal_period`.

## Conventions

- The two averages are started so that both reach their first value on the
  same bar. Because they may be different kinds, it is their warm-ups that are
  lined up, not their periods: a `tema` over 12 bars warms up more slowly than
  an `sma` over 26 and is the one that waits.
- Warm-up is the longer of the two warm-ups plus the signal average's.
- The default is `sma` for all three, which is what TA-Lib uses and is not the
  same thing as `macd`.
- Path dependence follows the averages chosen: with `sma`, `wma` or `trima`
  throughout there is none.

## Example

```python
import trendlib as tl
macd, signal, hist = tl.macdext(close, fast_period=12, slow_period=26, signal_period=9)
```

## References

- Gerald Appel, *Technical Analysis - Power Tools for Active Investors*, FT Press, 2005.
