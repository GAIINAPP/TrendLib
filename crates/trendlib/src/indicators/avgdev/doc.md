# Average Deviation

How far the last `period` values sit from their own average, on average. It is a
spread measure that, unlike the standard deviation, does not square the
distances, so a single outlier counts once rather than twice over.

## Formula

$$
\mathrm{AVGDEV}_t = \frac{1}{n}\sum_{i=t-n+1}^{t}\bigl|x_i - \bar{x}_t\bigr|
$$

## Conventions

- Lookback is $n - 1$.
- Not recursive: each row depends only on the last $n$ bars.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).
- The average is re-summed from the window each bar, because the next step
  subtracts it from values of the same size (`CONVENTIONS.md` section 1).

## Example

```python
import trendlib as tl

values = tl.avgdev(close, period=14)
```

## References

- Steven B. Achelis, Technical Analysis from A to Z, McGraw-Hill, 2000
