# Wilder Volatility System

J. Welles Wilder's Volatility System: a level a multiple of the average true
range away from the most extreme close of the current swing, which changes side
when a close crosses it.

## Formula

With $\mathrm{ATR}_n$ TrendLib's `atr` and $c = m\,\mathrm{ATR}_n(t)$, the swing has a
direction $\delta_t = \pm1$ and an extreme close $X_t$. On the first bar with an
average, $t_0 = n$: $\delta = 1$ if $C_{t_0} \ge C_0$, else $-1$, and $X = C_{t_0}$.
After it:

$$
(\delta_t, X_t) = \begin{cases}
(-1, C_t) & \delta_{t-1} = 1 \wedge C_t < V_{t-1}\\
(1, C_t) & \delta_{t-1} = -1 \wedge C_t > V_{t-1}\\
(1, \max(X_{t-1}, C_t)) & \delta_{t-1} = 1 \text{ otherwise}\\
(-1, \min(X_{t-1}, C_t)) & \delta_{t-1} = -1 \text{ otherwise}
\end{cases}
\qquad
V_t = X_t - \delta_t\,c
$$

`wilder_volatility` is $V_t$ and `wilder_volatility_direction` is $\delta_t$, where
$n$ is `period` and $m$ is `multiplier`.

## Conventions

- Wilder calls the level the stop-and-reverse point; TrendLib reports it as a
  level and a direction (D11).
- Wilder does not say how the first swing is chosen. It starts rising if the
  close on the first bar with an average true range is at least the first close,
  and falling if not.
- A close equal to the level does not change the side.
- Warm-up is `period` bars, the average true range's. An `int32` column warms up
  with `0`, not `NaN` (`CONVENTIONS.md` § 2).
- Path dependent: the swing in force depends on every bar since the first.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `ATR` and `MULT`, the swing followed bar by bar in NumPy (a
  transcription of the rule).

## Example

```python
import trendlib as tl
volatility, volatility_direction = tl.wilder_volatility(high, low, close)
```

## References

- J. Welles Wilder, New Concepts in Technical Trading Systems, Trend Research, 1978
