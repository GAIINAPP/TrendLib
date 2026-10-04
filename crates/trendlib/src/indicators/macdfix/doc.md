# MACD with Fixed Periods

MACD with the two averages fixed at 12 and 26 bars, leaving only the signal
length to choose. It is not the same line as `macd(fast_period=12,
slow_period=26)`: the smoothing constants are different numbers.

## Formula

$$
\mathrm{macd}_t = \mathrm{EMA}_{k=0.15}(x, 12)_t - \mathrm{EMA}_{k=0.075}(x, 26)_t
$$

$$
\mathrm{signal}_t = \mathrm{EMA}(\mathrm{macd}, m)_t \qquad
\mathrm{hist}_t = \mathrm{macd}_t - \mathrm{signal}_t
$$

where $x$ is `source` and $m$ is `signal_period`.

## Conventions

- The smoothing constants are the literals 0.075 and 0.15, not $2/(26+1) =
  0.074074$ and $2/(12+1) = 0.153846$. TA-Lib writes them that way, so
  `macdfix` and `macd(12, 26)` differ by a little under three points on the
  committed dataset. Both are kept, under the names TA-Lib gives them.
- The 12-bar average starts 14 bars late so both averages reach their first
  value on the same bar, as in `macd`.
- Warm-up is `25 + signal_period - 1`.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5);
  `unstable` applies.
- The signal line uses the ordinary $2/(m+1)$ constant.

## Example

```python
import trendlib as tl
macd, signal, hist = tl.macdfix(close, signal_period=9)
```

## References

- Gerald Appel, *Technical Analysis - Power Tools for Active Investors*, FT Press, 2005.
