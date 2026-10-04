# KDJ

The slow stochastic with Wilder's smoothing, plus a third line that overshoots both.

## Formula

$K$ and $D$ are `stoch`'s two lines with `rma` smoothing by default, and

$$
J_t = 3K_t - 2D_t
$$

## Conventions

- Warm-up is `fastk_period - 1` plus both averages' own.
- The J line is a linear combination of the other two, so it reaches outside
  0 to 100 whenever they diverge; that is the point of it.
- All three columns start on the same row.
- The default averages are `rma`, where `stoch`'s are `sma`. That is the only
  difference between the K and D lines of the two.
- `rma` is recursive, so the result is path dependent
  (`CONVENTIONS.md` § 5) at the default.

## Example

```python
import trendlib as tl
kdj_k, kdj_d, kdj_j = tl.kdj(high, low, close)
```

## References

- George C. Lane, Lane's Stochastics, Technical Analysis of Stocks and Commodities, May-June 1984
