# Williams Accumulation/Distribution

A running total of how much ground each bar took, measured from the previous close rather than from its own open.

## Formula

$$
\mathrm{step}_t =
\begin{cases}
C_t - \min(L_t, C_{t-1}) & C_t > C_{t-1}\\
C_t - \max(H_t, C_{t-1}) & C_t < C_{t-1}\\
0 & \text{otherwise}
\end{cases}
\qquad
\mathrm{wad}_t = \sum_{i \le t} \mathrm{step}_i
$$

## Conventions

- Lookback 0: the first bar starts the running total at zero.
- Path dependent (`CONVENTIONS.md` § 5): the value carries every bar before it,
  so a run started later begins its own total from zero.
- A bar that closed exactly where the last one did contributes nothing, whatever
  its range.
- Volume is not an input despite the name; this measures price alone.

## Example

```python
import trendlib as tl
wad = tl.wad(high, low, close)
```

## References

- Larry Williams, The Secret of Selecting Stocks for Immediate and Substantial Gains, Windsor Books, 1972
