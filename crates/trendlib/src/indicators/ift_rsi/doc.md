# Inverse Fisher Transform of RSI

finta's modified inverse Fisher transform of RSI: the RSI rescaled to about -5
to 5, weighted-averaged, and folded into -1 to 1 by its square, which pushes
readings towards the ends of the range.

## Formula

With $E_\alpha$ the adjusted exponential mean of everything so far, $U$ and
$D$ the rises and falls of the close:

$$
\text{RSI}_t = 100 - \frac{100}{1 + E_{1/r}(U)_t / E_{1/r}(D)_t} \qquad
v_t = 0.1\,(\text{RSI}_t - 50)
$$

$$
w_t = \frac{\sum_{k=1}^{m} k\, v_{t-m+k}}{m(m+1)/2} \qquad
\text{ift\_rsi}_t = \frac{w_t^2 - 1}{w_t^2 + 1}
$$

where $r$ is `rsi_period` and $m$ is `wma_period`.

## Conventions

- The transform squares the average rather than exponentiating it, as finta
  does, so it reads -1 where the average is 0 and approaches 1 as the average
  leaves 0 in either direction; it is not Ehlers' $(e^{2x} - 1)/(e^{2x} + 1)$.
- Its RSI's averages are pandas' `ewm(adjust=True)`, finta's: each is the
  weighted mean of every value so far, the newest weighted 1 and each older one
  `1 - alpha` times the next, so it starts at the first value and never forgets
  one. That makes it path dependent: a slice gives different values from the
  same rows of the whole series.
- The RSI is finta's, on those averages, not Wilder's RSI that `rsi` computes;
  it starts on the second bar and the output `wma_period` bars after the first.
- Where the close has not moved over the whole history so far, the RSI is 0/0
  and the output NaN.
- Oracle F (`DECISIONS.md` D19): the golden files are `finta` 1.3's
  `TA.IFT_RSI`, test-only and never shipped.

## Example

```python
import trendlib as tl
ift_rsi = tl.ift_rsi(source)
```

## References

- John Ehlers, Cybernetic Analysis for Stocks and Futures, Wiley, 2004
- finta 1.3, finta.TA (oracle F)
