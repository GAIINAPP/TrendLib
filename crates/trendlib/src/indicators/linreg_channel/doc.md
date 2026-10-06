# Linear Regression Channel

The least-squares line through the last `period` values, read at the newest bar,
with bands a multiple of the standard deviation of the values about that line
above and below it.

## Formula

Fit $y = a + b\,j$ by least squares to $x_{t-n+1+j}$, $j = 0 \dots n-1$ (TrendLib's
`linearreg_intercept` and `linearreg_slope`), and with the residuals
$e_j = x_{t-n+1+j} - (a + b\,j)$,

$$
\text{middle}_t = a + b\,(n - 1) \qquad
\sigma_t = \sqrt{\frac{1}{n-1}\sum_{j=0}^{n-1} e_j^2}
\qquad
\text{upper}_t,\ \text{lower}_t = \text{middle}_t \pm d\,\sigma_t
$$

where $x$ is `source`, $n$ is `period` and $d$ is `deviation`.

## Conventions

- The middle line is `linearreg`. The deviation is of the values about the
  fitted line, with $n - 1$ in the divisor, as TradingView's Linear Regression
  Channel computes it; it is not `stddev` of the values themselves.
- TradingView draws the channel once, over the last window; this gives the
  channel's end on every row, each from its own window.
- Residuals beyond about 1e154 overflow when squared and the bands read
  infinite, as `stddev`'s do.
- Warm-up is `period - 1` bars.
- Not path dependent.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `LINEARREG`, `LINEARREG_SLOPE`, `LINEARREG_INTERCEPT` and `MULT`, the
  residuals in NumPy.

## Example

```python
import trendlib as tl
channel_upper, channel_middle, channel_lower = tl.linreg_channel(source)
```

## References

- TradingView, Linear Regression Channel
