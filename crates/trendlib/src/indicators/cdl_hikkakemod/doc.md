# Modified Hikkake Pattern

The hikkake with two conditions added: the inside bar must itself sit inside
the bar before it, and that bar must have closed at the end of its own range in
the direction the break will be read as. Both make the setup rarer and the
reading stronger.

## Formula

A setup occurs at bar $t$ when bar $t-2$ sits inside bar $t-3$, bar $t-1$ sits
inside bar $t-2$, bar $t$ steps wholly to one side of bar $t-1$, and bar $t-2$
closed within the `near` threshold of the end of its own range on that side:

$$
\text{reading} =
\begin{cases}
+100 & H_t < H_{t-1},\ L_t < L_{t-1},\ C_{t-2} \le L_{t-2} + \text{near}\\
-100 & H_t > H_{t-1},\ L_t > L_{t-1},\ C_{t-2} \ge H_{t-2} - \text{near}
\end{cases}
$$

Within three bars of a setup, a close beyond the inside bar in the setup's own
direction, above $H_{t-1}$ for a positive reading or below $L_{t-1}$ for a
negative one, reads $\pm 200$ and closes the setup.

## Conventions

- Warm-up is 10 bars: five of pattern over the five-bar `near` average that
  qualifies the second of them. The state machine starts three bars earlier,
  so a setup from before the first reported row can still be confirmed in it.
- The sign is inverted against the break, as in `cdl_hikkake`.
- One nested inside bar is not enough: without the second, the pattern fires
  eighteen times on the committed dataset instead of three.
- A new setup replaces any pending one, confirmed or not.
- Path dependent (`CONVENTIONS.md` § 5): a run started later carries a
  different setup into its first rows.
- The only values are `0`, `±100` and `±200`.

## Example

```python
import trendlib as tl
found = tl.cdl_hikkakemod(open, high, low, close)
```

## References

- Daniel L. Chesler, "Quantifying Support and Resistance", *Technical Analysis of Stocks and Commodities*, March 2003.
