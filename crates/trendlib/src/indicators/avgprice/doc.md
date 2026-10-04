# Average Price

The plain average of a bar's open, high, low and close. It gives one number for
the bar without favouring where it opened or closed.

## Formula

$$
\mathrm{AVGPRICE}_t = \frac{o_t + h_t + l_t + c_t}{4}
$$

## Conventions

- Lookback is 0: every bar has a value.
- Not recursive: each row depends only on its own bar.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

values = tl.avgprice(df)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
