# Typical Price

The average of a bar's high, low and close. It is the summary several other
indicators are built on, including the commodity channel index and vwap.

## Formula

$$
\mathrm{TYPPRICE}_t = \frac{h_t + l_t + c_t}{3}
$$

## Conventions

- Lookback is 0: every bar has a value.
- Not recursive: each row depends only on its own bar.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

values = tl.typprice(df)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
