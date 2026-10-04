# Hikkake Pattern

An inside bar that is broken in one direction, read as a move in the other. The
pattern scores once when the break happens and again if a close within the next
three bars goes back through the inside bar.

## Formula

A setup occurs at bar $t$ when bar $t-1$ sits inside bar $t-2$
($H_{t-1} < H_{t-2}$ and $L_{t-1} > L_{t-2}$) and bar $t$ steps wholly to one
side of it:

$$
\text{reading} =
\begin{cases}
+100 & H_t < H_{t-1} \text{ and } L_t < L_{t-1}\\
-100 & H_t > H_{t-1} \text{ and } L_t > L_{t-1}
\end{cases}
$$

Within three bars of a setup, a close beyond the inside bar in the setup's own
direction, above $H_{t-1}$ for a positive reading or below $L_{t-1}$ for a
negative one, reads $\pm 200$ and closes the setup.

## Conventions

- Warm-up is 5 bars. The state machine starts three bars earlier, so a setup
  from before the first reported row can still be confirmed in it.
- The sign is inverted against the break: a bar stepping **down** out of the
  inside bar reads `+100`. That is the pattern's point, and it is TA-Lib's
  sign.
- A new setup replaces any pending one, confirmed or not.
- Path dependent (`CONVENTIONS.md` § 5): a run started later carries a
  different setup into its first rows.
- The only values are `0`, `±100` and `±200`.

## Example

```python
import trendlib as tl
found = tl.cdl_hikkake(open, high, low, close)
```

## References

- Daniel L. Chesler, "Quantifying Support and Resistance", *Technical Analysis of Stocks and Commodities*, March 2003.
