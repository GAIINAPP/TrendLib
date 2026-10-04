# Variance

The mean squared distance of the last `period` values from their own average.
It is the square of `stddev` and carries the units of the series squared, which
is why the standard deviation is the one usually plotted.

## Formula

$$
\mathrm{VAR}_t = \frac{1}{n}\sum_{i=t-n+1}^{t}\bigl(x_i - \bar{x}_t\bigr)^2
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Population variance: divided by $n$, not $n - 1$, as TA-Lib divides it.
- Lookback is $n - 1$.
- Squaring a distance overflows before the input does, so a series near the
  top of the float64 range gives an infinity. The spec carries the
  `nan_inf_output` flag to say so.
- **`nbdev` is accepted and has no effect.** TA-Lib declares the parameter for
  this function and then ignores it, so the same value comes back whatever is
  passed; TrendLib matches that rather than inventing a meaning for it (D6).
  `stddev` is the function where `nbdev` does something.
- Computed as the sum of squared distances from the window's own mean, for the
  accuracy reason given in `stddev`'s conventions.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

spread = tl.var(close, period=5)
```

## References

- Standard population statistics; TA-Lib exposes it as VAR.
