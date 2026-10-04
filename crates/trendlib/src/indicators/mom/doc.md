# Momentum

The change over a fixed number of bars, in the units of the series itself.

## Formula

$$
\mathrm{MOM}_t = x_t - x_{t-n}
$$

## Conventions

- Lookback is $n$: the comparison needs a bar that many places back.
- Not recursive: each row depends on two bars and nothing else, so a slice
  gives the same answer as the full series.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

change = tl.mom(close, period=10)
```

## References

- Martin J. Pring, Technical Analysis Explained, McGraw-Hill, 2014, chapter 13
