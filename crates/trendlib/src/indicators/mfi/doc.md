# Money Flow Index

How much of the money that changed hands recently did so on bars that closed
higher, as a percentage. It is the RSI's question asked of volume rather than
of price alone.

## Formula

With $\mathrm{TP}_t = (H_t + L_t + C_t)/3$ and flow $\mathrm{TP}_t \cdot V_t$,

$$
P_t = \sum_{\substack{i \in [t-n+1,\,t] \\ \mathrm{TP}_i > \mathrm{TP}_{i-1}}} \mathrm{TP}_i V_i
\qquad
N_t = \sum_{\substack{i \in [t-n+1,\,t] \\ \mathrm{TP}_i < \mathrm{TP}_{i-1}}} \mathrm{TP}_i V_i
$$

$$
\mathrm{mfi}_t = 100 \cdot \frac{P_t}{P_t + N_t}
$$

where $n$ is `period`.

## Conventions

- Warm-up is `period` bars, one more than the number of changes summed, since
  the first bar has nothing to be compared against.
- A bar whose typical price is unchanged adds to neither side rather than being
  split between them.
- A window with no flow at all, whether because volume was zero or because
  nothing moved, gives `0.0`. The test is on the exact total.
- Money flow is a price multiplied by a volume, so a series far enough from
  ordinary prices overflows: above about `1e154` on both the flow is infinite
  and the share is `NaN`.
- Not path dependent: the value depends on `period + 1` bars and nothing else.

## Example

```python
import trendlib as tl
flow = tl.mfi(high, low, close, volume, period=14)
```

## References

- Gene Quong and Avrum Soudack, "Volume-Weighted RSI: Money Flow", *Technical Analysis of Stocks and Commodities*, March 1989.
