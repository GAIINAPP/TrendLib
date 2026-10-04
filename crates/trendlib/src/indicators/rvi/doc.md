# Relative Volatility Index

The RSI's question asked of volatility: how much of it arrived on bars that closed up.

## Formula

With $\sigma_t$ the standard deviation over `stddev_period` bars,

$$
U_t = \begin{cases}\sigma_t & x_t > x_{t-1}\\ 0 & \text{otherwise}\end{cases}
\qquad
D_t = \begin{cases}\sigma_t & x_t < x_{t-1}\\ 0 & \text{otherwise}\end{cases}
$$

$$
\mathrm{rvi}_t = 100 \cdot \frac{\mathrm{RMA}(U, n)_t}{\mathrm{RMA}(U, n)_t + \mathrm{RMA}(D, n)_t}
$$

## Conventions

- Warm-up is `stddev_period - 1` plus `period - 1`: the deviations start as
  soon as their window is full and Wilder's average then takes `period` of
  them.
- A bar that closed exactly where the last one did contributes to neither side.
- A stretch with no volatility at all reads `50.0`: neither side has anything,
  so the share is even. The test is exact.
- The deviation squares its distances, so a series above about `1e154`
  overflows and the row is non-finite, as `stddev` does.
- Wilder's average is recursive, so the result is path dependent
  (`CONVENTIONS.md` § 5); `unstable` applies.

## Example

```python
import trendlib as tl
rvi = tl.rvi(source)
```

## References

- Donald Dorsey, The Relative Volatility Index, Technical Analysis of Stocks and Commodities, June 1993
