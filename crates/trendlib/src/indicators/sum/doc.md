# Summation

The total of the last `period` values. It is the rolling sum the simple moving
average divides by its period.

## Formula

$$
\mathrm{SUM}_t = \sum_{i=t-n+1}^{t} x_i
$$

## Conventions

- Lookback is $n - 1$.
- Not recursive: each row depends only on the last $n$ bars.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).
- The window is re-summed each bar rather than advanced, for the reason in
  `CONVENTIONS.md` section 1.

## Example

```python
import trendlib as tl

values = tl.sum(close, period=30)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
