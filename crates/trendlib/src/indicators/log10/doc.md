# Vector Base 10 Logarithm

The base ten logarithm of the input. Zero gives negative infinity and a negative
input gives NaN.

## Formula

$$
\log_{10}(x_t)
$$

## Conventions

- Lookback is 0: every bar has a value.
- The function is applied as IEEE-754 defines it and nothing is guarded,
  so an input outside the domain gives `NaN` and an overflow gives an
  infinity, exactly as TA-Lib leaves them. The spec carries the
  `nan_inf_output` flag to say so.
- Not recursive: each row depends only on its own bar.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid
  bar raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

values = tl.log10(source)
```

## References

- ISO/IEC 9899, the C standard library, section 7.12 (mathematical functions)
