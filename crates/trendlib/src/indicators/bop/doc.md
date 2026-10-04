# Balance of Power

How much of a bar's range the move from open to close accounted for, from -1 when
it closed at the low having opened at the high, to +1 for the reverse.

## Formula

$$
\mathrm{BOP}_t = \frac{c_t - o_t}{h_t - l_t}
$$

## Conventions

- Lookback is 0: every bar has a value.
- A bar whose high equals its low reports 0 rather than dividing by zero.
- Not recursive: each row depends only on its own bar.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

balance = tl.bop(df)
```

## References

- Igor Livshin, Balance of Power, Technical Analysis of Stocks and Commodities, 2001
