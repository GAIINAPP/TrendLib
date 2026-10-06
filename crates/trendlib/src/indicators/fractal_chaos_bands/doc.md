# Fractal Chaos Bands

Bill Williams' fractals as bands: the high of the latest confirmed swing high
and the low of the latest confirmed swing low, each held until the next one is
confirmed.

## Formula

Bar $k$ is a swing high when $H_k > H_i$ for every other $i \in [k-l,\ k+r]$,
known at bar $k + r$; a swing low is the same with $L_k < L_i$. Then

$$
\text{upper}_t = H_k \text{ for the latest swing high with } k + r \le t
\qquad
\text{lower}_t = L_k \text{ for the latest swing low with } k + r \le t
$$

where $l$ is `left_bars` and $r$ is `right_bars`.

## Conventions

- The swing points are `fractal`'s: strict on both sides, so a bar that ties a
  neighbour is not one, and each is known `right_bars` after it happens. Nothing
  is reported in the past.
- Each band is `NaN` until the first swing of its side is confirmed, which can
  be long after the warm-up, and stays `NaN` on a series with none.
- Warm-up is `left_bars + right_bars` bars.
- Path dependent: a band can hold a swing from arbitrarily far back.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate this formula through
  TA-Lib's `MAX` and `MIN` (the swing tests), the bands carried forward in
  NumPy.

## Example

```python
import trendlib as tl
chaos_upper, chaos_lower = tl.fractal_chaos_bands(high, low)
```

## References

- Bill Williams, Trading Chaos, Wiley, 1995
