# Beta

How far the first series tends to move when the second does, fitted over their returns.

## Formula

With $r^{(0)}_t = \dfrac{x_t - x_{t-1}}{x_{t-1}}$ and $r^{(1)}$ the same for the
second series,

$$
\beta_t = \frac{n\sum r^{(0)} r^{(1)} - \sum r^{(0)} \sum r^{(1)}}
{n\sum \left(r^{(0)}\right)^2 - \left(\sum r^{(0)}\right)^2}
$$

over the `period` returns ending at $t$.

## Conventions

- Warm-up is `period` bars, one more than the number of returns fitted.
- The fit is over returns, not prices, so the result does not change when
  either series is rescaled.
- A window in which the first series never moved reads `0.0`. The test is
  exact.
- Returns are bounded whatever the prices are, so this does not overflow where
  `correl` does.
- Not path dependent.

## Example

```python
import trendlib as tl
beta = tl.beta(source0, source1)
```

## References

- TA-Lib, ta_BETA.c
