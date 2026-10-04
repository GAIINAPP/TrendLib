# Linear Regression Angle

The slope of the fitted line expressed as an angle in degrees, between -90 and
90.

## Formula

$$
\mathrm{angle}_t = \arctan(m_t) \cdot \frac{180}{\pi}
$$

where $m$ is the fitted slope over `period` bars.

## Conventions

- Warm-up is `period - 1` bars.
- The angle depends on the units of the series, because the arctangent is taken
  of a slope measured in price per bar. A chart's visual angle also depends on
  its axes, so the two need not agree.
- A window whose values are all equal gives `-0.0`, which compares equal to
  `0.0`.
- Not path dependent.

## Example

```python
import trendlib as tl
angle = tl.linearreg_angle(close, period=14)
```

## References

- TA-Lib, `ta_LINEARREG_ANGLE.c`.
