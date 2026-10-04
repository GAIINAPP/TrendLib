# Market Facilitation Index

How much ground a bar covered for each unit of volume that traded in it.

## Formula

$$
\mathrm{marketfi}_t = \frac{H_t - L_t}{V_t}
$$

## Conventions

- Lookback 0: the value depends on one bar and nothing else.
- A bar with no volume gives `0.0`: there was no trading for the range to be
  divided among. The test is on the exact volume, as TA-Lib's is.
- Not path dependent.

## Example

```python
import trendlib as tl
marketfi = tl.marketfi(high, low, volume)
```

## References

- Bill Williams, Trading Chaos, Wiley, 1995
