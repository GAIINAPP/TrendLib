# Rate of Change

How far the series has moved, as a percentage, compared with where it stood a
fixed number of bars earlier. It says nothing about the path taken in between:
only the two endpoints matter, so a round trip back to the starting value reads
as zero.

## Formula

$$
\mathrm{ROC}_t = 100\,\frac{x_t - x_{t-n}}{x_{t-n}}
$$

where $x$ is `source` and $n$ is `period`.

## Conventions

- Lookback is $n$: the comparison needs a bar that many places back, so the
  first $n$ rows have nothing to compare against.
- A value of exactly zero $n$ bars back leaves the result undefined, and the
  row is `NaN` rather than an infinity. Prices are not required to be positive
  (`CONVENTIONS.md` section 3), so this is reachable on a synthetic series.
- Not recursive: each row depends on two bars and nothing else, so a slice
  gives the same answer as the full series.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

change = tl.roc(close, period=10)
```

## References

- Martin J. Pring, *Technical Analysis Explained*, McGraw-Hill, 2014,
  chapter 13.
