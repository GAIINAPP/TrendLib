# Vertical Horizontal Filter

How much of the distance travelled over the window the series actually covered, between 0 and 1.

## Formula

$$
\mathrm{vhf}_t = \frac{\max_{i} x_i - \min_{i} x_i}{\sum_{i} |x_i - x_{i-1}|}
$$

over the `period` bars ending at $t$.

## Conventions

- Warm-up is `period` bars.
- A window that did not move at all reads `0.0`, as TA-Lib does. The test is
  exact.
- Close to `er`, but measured against the window's whole range rather than
  against its endpoints, so a series that doubled back still reads high.
- Not path dependent.

## Example

```python
import trendlib as tl
vhf = tl.vhf(source)
```

## References

- Adam White, Vertical Horizontal Filter, Futures, August 1991
