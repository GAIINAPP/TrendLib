# Rate of Change Ratio

The ratio of the latest value to the one a fixed number of bars back. A reading
of 1 means no change.

## Formula

$$
\mathrm{ROCR}_t = \frac{x_t}{x_{t-n}}
$$

## Conventions

- Lookback is $n$: the comparison needs a bar that many places back.
- A value of exactly zero $n$ bars back leaves the result undefined, and the
  row is `NaN` rather than an infinity.
- Not recursive: each row depends on two bars and nothing else, so a slice
  gives the same answer as the full series.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

change = tl.rocr(close, period=10)
```

## References

- Martin J. Pring, Technical Analysis Explained, McGraw-Hill, 2014, chapter 13
