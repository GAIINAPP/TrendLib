# Triple Exponential Average

The bar-to-bar percentage change of a triply smoothed exponential average.
Three passes of smoothing remove most of what moves for only a bar or two, so
what is left is the rate at which the longer trend is changing.

## Formula

$$
e^{(1)} = \mathrm{EMA}(x, n) \qquad
e^{(2)} = \mathrm{EMA}(e^{(1)}, n) \qquad
e^{(3)} = \mathrm{EMA}(e^{(2)}, n)
$$

$$
\mathrm{trix}_t = 100 \cdot \frac{e^{(3)}_t - e^{(3)}_{t-1}}{e^{(3)}_{t-1}}
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Warm-up is `3 * (period - 1) + 1` bars: three exponential stages and one more
  bar to compare against.
- A triple average of exactly zero leaves the change undefined and the row is
  `NaN` rather than an infinity, the same reading `rocp` takes.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5) and the
  early rows carry the seed; `unstable` applies.
- This is the percentage change, already multiplied by 100.

## Example

```python
import trendlib as tl
rate = tl.trix(close, period=30)
```

## References

- Jack K. Hutson, "Good TRIX", *Technical Analysis of Stocks and Commodities*, July 1983.
