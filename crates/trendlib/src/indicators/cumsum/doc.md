# Cumulative Sum

The running total of the input from the first valid bar onwards.

## Formula

$$
\mathrm{CUMSUM}_t = \sum_{i=0}^{t} x_i
$$

## Conventions

- Lookback is 0: the first bar already has a value.
- **Path dependent.** The total carries every bar before it, so the same rows
  computed from a later starting point differ by a constant
  (`CONVENTIONS.md` section 5).
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

values = tl.cumsum(source)
```

## References

- Standard running total; TA-Lib exposes it as CUMSUM
