# Weighted Close Price

A bar summarised with its close counted twice, on the view that where trading
finished says more than where it reached.

## Formula

$$
\mathrm{WCLPRICE}_t = \frac{h_t + l_t + 2c_t}{4}
$$

## Conventions

- Lookback is 0: every bar has a value.
- Not recursive: each row depends only on its own bar.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

values = tl.wclprice(df)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
