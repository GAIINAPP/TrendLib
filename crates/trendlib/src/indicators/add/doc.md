# Vector Addition

The two inputs added bar by bar.

## Formula

$$
x_t + y_t
$$

## Conventions

- Lookback is 0: every bar has a value.
- Not recursive: each row depends only on its own bar.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

values = tl.add(source0, source1)
```

## References

- ISO/IEC 9899, the C standard library, section 7.12 (mathematical functions)
