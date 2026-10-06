# Elder Impulse System

Alexander Elder's Impulse System: whether an exponential average of the close
and the MACD histogram both rose on this bar (1), both fell (-1), or neither
(0).

## Formula

With $E_t$ the `ema_period` exponential average of the close and $h_t$ the
histogram of TrendLib's `macd` at `fast_period`, `slow_period` and `signal_period`,

$$
\text{elder\_impulse}_t = \begin{cases}
1 & E_t > E_{t-1} \wedge h_t > h_{t-1}\\
-1 & E_t < E_{t-1} \wedge h_t < h_{t-1}\\
0 & \text{otherwise}
\end{cases}
$$

## Conventions

- Elder colours the bars green, red and blue; the output is 1, -1 and 0, and
  names no action.
- Both comparisons are strict, so a bar on which either is unchanged reads 0.
- Warm-up is one bar past the later of the average and the histogram:
  `max(ema_period - 1, macd lookback) + 1`, 34 at the defaults. An `int32`
  column warms up with `0`, not `NaN` (`CONVENTIONS.md` § 2).
- Recursive through the averages.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `EMA` and `MACD`, the comparisons in NumPy.

## Example

```python
import trendlib as tl
elder_impulse = tl.elder_impulse(source)
```

## References

- Alexander Elder, Come Into My Trading Room, Wiley, 2002
