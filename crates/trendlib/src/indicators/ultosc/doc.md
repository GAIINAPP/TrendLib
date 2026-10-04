# Ultimate Oscillator

How much of the recent trading range the close claimed, measured over three
window lengths at once and blended four to two to one. Using three lengths
makes it respond to short swings without being driven by them.

## Formula

With $\mathrm{BP}_t = C_t - \min(L_t, C_{t-1})$ and
$\mathrm{TR}_t = \max(H_t, C_{t-1}) - \min(L_t, C_{t-1})$,

$$
A_t(n) = \frac{\sum_{i=t-n+1}^{t} \mathrm{BP}_i}{\sum_{i=t-n+1}^{t} \mathrm{TR}_i}
$$

$$
\mathrm{ultosc}_t = 100 \cdot \frac{4 A_t(n_1) + 2 A_t(n_2) + A_t(n_3)}{7}
$$

where $n_1$, $n_2$ and $n_3$ are `period1`, `period2` and `period3`.

## Conventions

- The weights follow the argument order, not the lengths: `period1` is always
  weighted four times even if it is the longest of the three.
- Warm-up is the longest of the three periods, which is one more bar than the
  number of ranges summed, since the first bar has no close to reach back to.
- A window whose true ranges sum to zero gives `0.0` for that term. The test is
  on the exact sum.
- Not path dependent.

## Example

```python
import trendlib as tl
blended = tl.ultosc(high, low, close, period1=7, period2=14, period3=28)
```

## References

- Larry Williams, "The Ultimate Oscillator", *Technical Analysis of Stocks and Commodities*, August 1985.
