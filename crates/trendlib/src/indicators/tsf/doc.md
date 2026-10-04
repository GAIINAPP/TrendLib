# Time Series Forecast

The fitted least squares line carried one bar past the end of the window: what
the line says the next value would be if the window's trend continued exactly.

## Formula

With $m$ and $b$ the slope and intercept fitted over the last $n$ values,

$$
\mathrm{tsf}_t = b + m\,n
$$

where $n$ is `period`. `linearreg` is the same line at $n - 1$, so `tsf` leads
it by exactly one slope.

## Conventions

- Warm-up is `period - 1` bars.
- The value sits at row `t`, not at row `t + 1`: it is a description of the
  window ending at `t`, not a value placed in the future.
- Not path dependent.

## Example

```python
import trendlib as tl
forecast = tl.tsf(close, period=14)
```

## References

- TA-Lib, `ta_TSF.c`.
