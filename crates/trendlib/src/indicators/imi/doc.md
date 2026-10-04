# Intraday Momentum Index

The RSI's question asked of the bodies alone: how much of the recent open-to-close movement was upward.

## Formula

$$
\mathrm{imi}_t = 100 \cdot
\frac{\sum_{i} \max(C_i - O_i, 0)}{\sum_{i} |C_i - O_i|}
$$

over the `period` bars ending at $t$.

## Conventions

- Warm-up is `period - 1` bars.
- This compares each bar's close with its own open, not with the previous
  close, so overnight gaps do not enter it at all.
- A window in which every bar closed where it opened reads `50.0`: neither side
  won anything, so the share is even. The test is on the exact total.
- Not path dependent.

## Example

```python
import trendlib as tl
imi = tl.imi(open, close)
```

## References

- Tushar S. Chande and Stanley Kroll, The New Technical Trader, Wiley, 1994
