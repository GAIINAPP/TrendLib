# Fractal

Marks the bar at which a swing high or low became confirmed by the bars on both sides of it.

## Formula

A swing high is confirmed at bar $t$ when

$$
H_{t-r} > H_i \quad \text{for every } i \in [t-r-l,\ t],\ i \ne t-r
$$

where $l$ is `left_bars` and $r$ is `right_bars`. A swing low is the same with
the minimum of the lows.

## Conventions

- Warm-up is `left_bars + right_bars` bars.
- The mark sits on the bar where the swing became **confirmed**, which is
  `right_bars` after the swing itself. Nothing is reported in the past.
- The comparison is strict: a bar that merely ties the ones around it is not a
  swing, so a flat stretch marks nothing.
- The output is `100` or `0`; an `int32` column warms up with `0`, not `NaN`
  (`CONVENTIONS.md` § 2).
- Not path dependent.

## Example

```python
import trendlib as tl
fractal_swing_high, fractal_swing_low = tl.fractal(high, low)
```

## References

- Bill Williams, Trading Chaos, Wiley, 1995
