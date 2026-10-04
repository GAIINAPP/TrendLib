# Plus Directional Movement

How much ground price has gained on the upside, accumulated with Wilder's
smoothing. Only the larger of the two edge movements counts on any bar, so a bar
inside the previous one contributes nothing.

## Formula

$$
\mathrm{+DM}_t = \begin{cases} h_t - h_{t-1} & h_t - h_{t-1} > l_{t-1} - l_t \text{ and } > 0\\ 0 & \text{otherwise}\end{cases}
$$

## Conventions

- Lookback is $n - 1$: one bar is consumed before the first movement exists,
  and the running total then needs $n - 1$ more.
- Accumulated as a running total, not an average: the seed is the plain sum of
  the first $n$ movements and each later bar gives
  $\mathrm{prev} - \mathrm{prev}/n + \mathrm{movement}$.
- Recursive, so early rows carry a trace of the seed (D9).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

movement = tl.plus_dm(df, period=14)
```

## References

- J. Welles Wilder Jr., New Concepts in Technical Trading Systems, Trend Research, 1978
