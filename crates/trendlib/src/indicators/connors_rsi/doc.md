# Connors RSI

Larry Connors' composite of three 0-to-100 readings: a short RSI of the close,
an RSI of how many bars in a row the close has risen or fallen, and the percent
rank of today's change among recent ones.

## Formula

The streak counts bars in a row the close has moved one way, from bar 1:

$$
s_t = \begin{cases}
\max(s_{t-1}, 0) + 1 & C_t > C_{t-1}\\
\min(s_{t-1}, 0) - 1 & C_t < C_{t-1}\\
0 & C_t = C_{t-1}
\end{cases}
\qquad s_0 = 0
$$

With $r_t = 100\,(C_t / C_{t-1} - 1)$ the one-bar change,

$$
\text{connors\_rsi}_t = \frac13\Big(\mathrm{RSI}_a(C)_t + \mathrm{RSI}_b(s)_t
+ \frac{100}{m}\,\#\{\,i \in [t-m,\ t-1] : r_i < r_t\,\}\Big)
$$

where $a$ is `rsi_period`, $b$ is `streak_period`, $m$ is `rank_period` and RSI is
TrendLib's `rsi` (Wilder's).

## Conventions

- The streak starts at bar 1, the first bar with a change, counting from 0
  before it; its RSI therefore starts at `1 + streak_period`.
- The percent rank is `percentrank`'s: today's change against the `rank_period`
  changes before it, strictly below, so a flat stretch reads 0.
- The one-bar change is $100\,(C_t / C_{t-1} - 1)$, written the way TA-Lib's
  `ROC` computes it, and 0 after a zero close.
- Warm-up is the longest of the three: `max(rsi_period, 1 + streak_period, 1 +
  rank_period)`, 101 at the defaults.
- Recursive through the two Wilder averages.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `RSI` (of the close and of the streak) and `ROC`, the streak and the
  rank counted in NumPy.

## Example

```python
import trendlib as tl
connors_rsi = tl.connors_rsi(source)
```

## References

- Larry Connors, Cesar Alvarez and Matt Radtke, An Introduction to ConnorsRSI, Connors Research, 2012
