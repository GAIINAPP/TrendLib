# Coppock Curve

A weighted average of two rates of change taken over different spans.

## Formula

$$
\mathrm{coppock}_t = \mathrm{WMA}\big(\mathrm{roc}(x, n_1) + \mathrm{roc}(x, n_2),\ m\big)_t
$$

where both rates of change are percentages.

## Conventions

- Warm-up is the longer of the two lags plus the weighted average's own.
- A value of exactly zero that far back makes a rate of change non-finite
  rather than guarded, as in `roc`.
- The two rates are added, not averaged, so the result is roughly twice the
  size of either.
- Not path dependent.

## Example

```python
import trendlib as tl
coppock = tl.coppock(source)
```

## References

- Edwin Sedgwick Coppock, Practical Relative Strength Charting, Trendex, 1962
