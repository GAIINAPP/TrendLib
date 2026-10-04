# Linear Regression Slope

The slope of a straight line fitted by least squares to the last `period`
values: how much the fitted line rises or falls per bar, in the units of the
series.

## Formula

With $x$ running $0$ to $n-1$ over the window and $y$ the values in it,

$$
\mathrm{slope}_t = \frac{n\sum xy - \sum x \sum y}{n \sum x^2 - \left(\sum x\right)^2}
$$

where $n$ is `period`.

## Conventions

- Warm-up is `period - 1` bars.
- A window whose values are all equal gives `-0.0`, which is what TA-Lib
  returns; it compares equal to `0.0`.
- The value is in units of `source` per bar, so it scales with the price and is
  not comparable across instruments. `linearreg_angle` is the same number read
  as a direction instead.
- Not path dependent.

## Example

```python
import trendlib as tl
slope = tl.linearreg_slope(close, period=14)
```

## References

- TA-Lib, `ta_LINEARREG_SLOPE.c`.
