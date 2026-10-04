# Linear Regression

The value at the newest bar of a straight line fitted by least squares to the
last `period` values. It follows the series as a moving average does, but
leans into the direction the window is already travelling rather than lagging
behind it.

## Formula

With $x$ running $0$ to $n-1$ over the window and $y$ the values in it,

$$
m = \frac{n\sum xy - \sum x \sum y}{n \sum x^2 - \left(\sum x\right)^2} \qquad
b = \frac{\sum y - m \sum x}{n}
$$

$$
\mathrm{linearreg}_t = b + m\,(n - 1)
$$

where $n$ is `period`.

## Conventions

- Warm-up is `period - 1` bars.
- The sums are taken over the window itself each bar rather than carried
  forward, so a long run cannot accumulate drift.
- A window whose values are all equal gives a slope of `-0.0` and the value
  itself, which is what TA-Lib returns.
- Not path dependent: the value at a row depends only on the `period` bars
  ending there.

## Example

```python
import trendlib as tl
fitted = tl.linearreg(close, period=14)
```

## References

- TA-Lib, `ta_LINEARREG.c`, least squares fit over a rolling window.
