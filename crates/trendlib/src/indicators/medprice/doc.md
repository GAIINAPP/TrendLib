# Median Price

The midpoint between a bar's high and low. It describes where the bar sat
without regard to where trading began or ended inside it.

## Formula

$$
\mathrm{MEDPRICE}_t = \frac{h_t + l_t}{2}
$$

## Conventions

- Lookback is 0: every bar has a value.
- Not recursive: each row depends only on its own bar.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

values = tl.medprice(df)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
