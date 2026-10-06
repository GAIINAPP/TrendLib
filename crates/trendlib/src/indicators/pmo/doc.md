# Price Momentum Oscillator

Carl Swenlin's DecisionPoint Price Momentum Oscillator: the one-bar rate of
change smoothed twice with DecisionPoint's custom averages and scaled by ten,
with an exponential average of it as the signal line.

## Formula

With $r_t = 100\,(C_t / C_{t-1} - 1)$ and $E^{(k)}$ an exponential average with
multiplier $k$, seeded as described below,

$$
\text{pmo}_t = E^{(2/b)}\Big(10\, E^{(2/a)}(r)\Big)_t \qquad
\text{pmo\_signal}_t = E^{(2/(c+1))}(\text{pmo})_t
$$

where $a$ is `first_period`, $b$ is `second_period` and $c$ is `signal_period`.

## Conventions

- DecisionPoint's custom smoothing uses the multiplier $2/n$, not $2/(n+1)$,
  which makes it an exponential average of period $n - 1$. DecisionPoint does
  not publish a seed; each is seeded as TrendLib's `ema` of period $n - 1$ is,
  with the mean of its first $n - 1$ values.
- The signal line is an ordinary exponential average of `signal_period`.
- The one-bar change is $100\,(C_t / C_{t-1} - 1)$, written the way TA-Lib's
  `ROC` computes it, and 0 after a zero close.
- Warm-up is `first_period + second_period + signal_period - 4`, 61 at the
  defaults; both outputs start there.
- Recursive: early rows still carry a trace of the three seeds.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `ROC`, `EMA` (at periods $a - 1$, $b - 1$ and $c$) and `MULT`.

## Example

```python
import trendlib as tl
pmo, signal = tl.pmo(source)
```

## References

- Carl Swenlin, DecisionPoint Price Momentum Oscillator (PMO), DecisionPoint.com
- StockCharts ChartSchool, DecisionPoint Price Momentum Oscillator
