# MESA Adaptive Moving Average

An exponential average whose step follows how fast the dominant cycle's phase is turning.

## Formula

With $\phi_t = \arctan(Q_t / I_t)$ in degrees and
$\Delta_t = \max(\phi_{t-1} - \phi_t,\ 1)$,

$$
\alpha_t = \begin{cases}
f & \Delta_t \le 1\\
\max\!\left(\dfrac{f}{\Delta_t},\ s\right) & \text{otherwise}
\end{cases}
$$

$$
\mathrm{mama}_t = (1-\alpha_t)\,\mathrm{mama}_{t-1} + \alpha_t x_t
\qquad
\mathrm{fama}_t = \left(1-\tfrac{\alpha_t}{2}\right)\mathrm{fama}_{t-1} + \tfrac{\alpha_t}{2}\,\mathrm{mama}_t
$$

## Conventions

- Warm-up is 32 bars, as for the readings that start twelve bars in.
- Both averages start at zero rather than at the first bar, so the early rows
  climb towards the series from below. That is TA-Lib's seed, and the warm-up
  covers it.
- `fama` steps at half `mama`'s rate, so the two cross where the trend turns.
- `fast_limit` and `slow_limit` are each between 0.01 and 0.99; the oracle
  refuses anything outside that and so does this.
- Recursive throughout, so the result is path dependent
  (`CONVENTIONS.md` § 5); `unstable` applies.

## Example

```python
import trendlib as tl
mama, mama_fama = tl.mama(source)
```

## References

- John F. Ehlers, MESA Adaptive Moving Averages, Technical Analysis of Stocks and Commodities, September 2001
