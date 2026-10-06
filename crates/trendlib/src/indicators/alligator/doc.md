# Williams Alligator

Bill Williams' three smoothed moving averages of the median price, each drawn a
few bars ahead: the jaw, teeth and lips. Lines close together and intertwined,
or spread apart in order, are how the chart is commonly read.

## Formula

With $M_t = (H_t + L_t)/2$ the median price and $S^{(n)}$ Williams' smoothed
moving average of it,

$$
S^{(n)}_{n-1} = \frac1n \sum_{j=0}^{n-1} M_j \qquad
S^{(n)}_t = S^{(n)}_{t-1} + \frac{M_t - S^{(n)}_{t-1}}{n}
$$

$$
\text{jaw}_t = S^{(a)}_{t-u} \qquad \text{teeth}_t = S^{(b)}_{t-v} \qquad \text{lips}_t = S^{(c)}_{t-w}
$$

where $a, u$ are `jaw_period`, `jaw_shift`; $b, v$ are `teeth_period`,
`teeth_shift`; and $c, w$ are `lips_period`, `lips_shift`.

## Conventions

- Williams' smoothed moving average is Wilder's smoothing: the simple mean of
  the first `period` medians, then each bar moves it `1/period` of the way to
  the new median. It is written as that step, $S_t = S_{t-1} + (M_t -
  S_{t-1})/n$, so a flat median leaves it exactly flat.
- Each line is given as drawn at bar $t$: computed `shift` bars earlier, so a
  bar can be compared with the lines on its own row. The lines a chart draws
  beyond the last bar are the next `shift` rows, which are not known yet.
- Every output starts on the same row, the largest of `period - 1 + shift` over
  the three lines: 20 at the defaults.
- Recursive: early rows still carry a trace of the seed, as any Wilder average
  does.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `MEDPRICE` and `SMA` (the seed), with the smoothing step in NumPy.

## Example

```python
import trendlib as tl
jaw, teeth, lips = tl.alligator(high, low)
```

## References

- Bill Williams, Trading Chaos, Wiley, 1995
