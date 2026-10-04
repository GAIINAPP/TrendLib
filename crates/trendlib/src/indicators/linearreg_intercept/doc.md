# Linear Regression Intercept

Where the fitted line starts: its value at the oldest bar of the window, the
$b$ in $y = b + mx$.

## Formula

With $x$ running $0$ to $n-1$ over the window, $y$ the values in it and $m$ the
fitted slope,

$$
\mathrm{intercept}_t = \frac{\sum y - m \sum x}{n}
$$

where $n$ is `period`.

## Conventions

- Warm-up is `period - 1` bars.
- Read together with `linearreg_slope` this gives the whole line:
  `intercept + slope * k` is the fitted value `k` bars after the start of the
  window, which at `k = period - 1` is `linearreg` and at `k = period` is `tsf`.
- Not path dependent.

## Example

```python
import trendlib as tl
intercept = tl.linearreg_intercept(close, period=14)
```

## References

- TA-Lib, `ta_LINEARREG_INTERCEPT.c`.
