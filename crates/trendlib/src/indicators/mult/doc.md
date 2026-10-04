# Vector Multiplication

The two inputs multiplied bar by bar.

## Formula

$$
x_t \cdot y_t
$$

## Conventions

- Lookback is 0: every bar has a value.
- Two large finite inputs can multiply to an infinity, which is left as it
  falls out rather than guarded; the spec carries the `nan_inf_output` flag
  to say so.
- Not recursive: each row depends only on its own bar.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

values = tl.mult(source0, source1)
```

## References

- ISO/IEC 9899, the C standard library, section 7.12 (mathematical functions)
