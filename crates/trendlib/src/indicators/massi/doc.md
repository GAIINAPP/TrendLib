# Mass Index

How much the bars have been widening, measured by how far a smoothed range runs ahead of a twice-smoothed one.

## Formula

With $e^{(1)} = \mathrm{EMA}(H - L, n)$ and $e^{(2)} = \mathrm{EMA}(e^{(1)}, n)$,

$$
\mathrm{massi}_t = \sum_{i=t-m+1}^{t} \frac{e^{(1)}_i}{e^{(2)}_i}
$$

where $n$ is `fast_period` and $m$ is `slow_period`.

## Conventions

- Warm-up is `2 * (fast_period - 1) + slow_period - 1`.
- A series that never moves smooths to zero on both stages, and the ratio
  reads 1 there, so the sum is `slow_period`. The test is exact.
- The sum, not the average, is reported, so the value scales with
  `slow_period`: around 25 at the default.
- Recursive, so the result is path dependent (`CONVENTIONS.md` § 5).

## Example

```python
import trendlib as tl
massi = tl.massi(high, low)
```

## References

- Donald Dorsey, The Mass Index, Technical Analysis of Stocks and Commodities, June 1992
