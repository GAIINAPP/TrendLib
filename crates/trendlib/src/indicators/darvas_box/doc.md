# Darvas Box

Nicolas Darvas' boxes: a top that later highs have not exceeded and a bottom,
after it, that later lows have not undercut, each for `confirm_bars` bars. Price
leaving the box starts the next one.

## Formula

A box forms in two steps from a starting bar, with $c$ = `confirm_bars`.

1. The top $T$ is the highest high since the start. A bar whose high exceeds $T$
   becomes the new top and clears any bottom found so far.
2. The bottom $B$ is the lowest low among the bars after the top, kept with the
   bar $k_B$ it came from; a lower low replaces it.

The box $(T, B)$ is confirmed at the first bar $t$ with $t - k_B \ge c$. It is in
force until a bar with $H_t > T$ or $L_t < B$, which starts the next box with that
bar's high as its first top.

$$
\text{darvas\_top}_t,\ \text{darvas\_bottom}_t = (T, B) \text{ of the latest box confirmed at or before } t
$$

## Conventions

- A confirmed box is reported until the next one is confirmed, including on the
  bars after price has left it; compare the close with the box to see which side
  it is on.
- Both edges are `NaN` until the first box is confirmed, which can be long after
  the warm-up.
- A high equal to the top does not replace it, and a low equal to the bottom
  does not replace it, so ties confirm sooner.
- The bar that sets the top is not a candidate for the bottom; the bottom comes
  after it.
- Darvas also filtered boxes by volume and by new highs for the year; those are
  not part of the rule here.
- Warm-up is `confirm_bars + 1` bars, the earliest a box can be confirmed.
- Path dependent: which box is in force depends on every bar since the first.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  the rule followed bar by bar in NumPy, a transcription of it; no TA-Lib
  function computes a step.

## Example

```python
import trendlib as tl
top, bottom = tl.darvas_box(high, low)
```

## References

- Nicolas Darvas, How I Made $2,000,000 in the Stock Market, 1960
