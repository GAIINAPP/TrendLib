# Fractal Adaptive Moving Average

John Ehlers' Fractal Adaptive Moving Average: an exponential average of the
median price whose step is set by the fractal dimension of the last `period`
bars, fast when price trends and slow when it churns.

## Formula

With $h = \lfloor n/2 \rfloor$, $\mathrm{rng}_k(s)$ the highest high less the lowest low
over the $k$ bars ending at $s$, and $M_t = (H_t + L_t)/2$,

$$
N_1 = \frac{\mathrm{rng}_h(t)}{h} \qquad N_2 = \frac{\mathrm{rng}_h(t - h)}{h} \qquad
N_3 = \frac{\mathrm{rng}_n(t)}{n}
$$

$$
D_t = \frac{\ln(N_1 + N_2) - \ln N_3}{\ln 2} \ \text{ if } N_1, N_2, N_3 > 0,
\text{ else } D_{t-1}
\qquad \alpha_t = \min\big(1, \max(0.01,\ e^{-4.6 (D_t - 1)})\big)
$$

$$
F_{n-1} = M_{n-1} \qquad F_t = F_{t-1} + \alpha_t (M_t - F_{t-1})
$$

where $n$ is `period` and $D$ is 0 before it is first computed.

## Conventions

- The step is Ehlers' $\alpha M_t + (1 - \alpha) F_{t-1}$ rearranged, so a flat
  price leaves it exactly flat.
- Ehlers uses an even `period`. An odd one is accepted: each half is $\lfloor
  n/2 \rfloor$ bars and the oldest bar of the window counts only in $N_3$.
- Where a range is zero the previous dimension is kept, as Ehlers' code does;
  before any is computed it is 0, which makes $\alpha = 1$.
- Warm-up is `period - 1` bars, where the average starts at the median price.
- Recursive: early rows still carry a trace of the start.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `MAX`, `MIN` and `MEDPRICE`, the dimension and the step in NumPy.

## Example

```python
import trendlib as tl
frama = tl.frama(high, low)
```

## References

- John Ehlers, FRAMA - Fractal Adaptive Moving Average, Technical Analysis of Stocks & Commodities, 2005
