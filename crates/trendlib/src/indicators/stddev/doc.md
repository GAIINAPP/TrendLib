# Standard Deviation

How far the last `period` values sit from their own average, in the units of
the series. It is the spread Bollinger Bands are built from, and it counts a
distant value more heavily than `avgdev` does because the distances are
squared.

## Formula

$$
\sigma_t = \sqrt{\frac{1}{n}\sum_{i=t-n+1}^{t}\bigl(x_i - \bar{x}_t\bigr)^2}
\qquad \mathrm{STDDEV}_t = k\,\sigma_t
$$

where $x$ is `source`, $n$ is `period` and $k$ is `nbdev`.

## Conventions

- Population standard deviation: the sum of squared distances is divided by
  $n$, not $n - 1$, as TA-Lib divides it.
- Lookback is $n - 1$.
- Squaring a distance overflows before the input does, so a series near the
  top of the float64 range gives an infinity. The spec carries the
  `nan_inf_output` flag to say so.
- The variance is taken as the sum of squared distances from the window's own
  mean. The cheaper identity, the mean of the squares minus the square of the
  mean, drifts about 1.6e-10 from the oracle, which is outside this project's
  tolerance; the form used here measures 7e-12.
- `nbdev` scales the result and nothing else.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

spread = tl.stddev(close, period=5, nbdev=2.0)
```

## References

- Standard population statistics; TA-Lib exposes it as STDDEV.
