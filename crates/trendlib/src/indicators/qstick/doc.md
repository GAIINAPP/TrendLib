# Qstick

The average distance from open to close over the last few bars, positive while they are closing up.

## Formula

$$
\mathrm{qstick}_t = \frac{1}{n}\sum_{i=t-n+1}^{t} (C_i - O_i)
$$

## Conventions

- Warm-up is `period - 1` bars.
- The sign is the body's, so a run of white bars reads positive whatever the
  prices did between them.
- Not path dependent.

## Example

```python
import trendlib as tl
qstick = tl.qstick(open, close)
```

## References

- Tushar S. Chande and Stanley Kroll, The New Technical Trader, Wiley, 1994
