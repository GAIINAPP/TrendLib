# Ichimoku Kinko Hyo

Midpoints of the highest high and lowest low over three spans: the conversion
and base lines, and two leading spans drawn ahead of price, whose band is the
cloud. Price above both spans, a rising cloud or the lines crossing are how
the chart is commonly read.

## Formula

$$
\mathrm{mid}_n(t) = \tfrac12\Big(\max_{t-n < j \le t} H_j + \min_{t-n < j \le t} L_j\Big)
$$

$$
\text{tenkan}_t = \mathrm{mid}_{a}(t) \qquad
\text{kijun}_t = \mathrm{mid}_{b}(t)
$$

$$
\text{senkou\_a}_t = \tfrac12\big(\text{tenkan}_{t-d} + \text{kijun}_{t-d}\big) \qquad
\text{senkou\_b}_t = \mathrm{mid}_{c}(t - d)
$$

where $a$ is `tenkan_period`, $b$ is `kijun_period`, $c$ is `senkou_period` and
$d$ is `displacement`.

## Conventions

- The leading spans are given as drawn at bar $t$: computed `displacement` bars
  earlier, so a close can be compared with the cloud on its own row. The cloud
  a chart shows beyond the last bar is these outputs' next `displacement` rows,
  which are not known yet. TradingView draws them `displacement - 1` bars ahead;
  pass `displacement=25` to match it. A span as computed, before it is drawn
  ahead, is the output `displacement` rows later.
- The lagging span (chikou) is the close drawn `displacement` bars back. It is
  the close itself, and reporting it at the bar it is drawn on would need bars
  that have not happened, so it is not an output.
- Every output starts on the same row: `max(tenkan_period, kijun_period,
  senkou_period) - 1 + displacement`, 77 at the defaults.
- Oracle A (`DECISIONS.md` D20): the golden files evaluate these formulas
  through TA-Lib's `MIDPRICE`, `ADD` and `MULT`.
- Not path dependent: every value depends on a fixed window of bars.

## Example

```python
import trendlib as tl
tenkan, kijun, span_a, span_b = tl.ichimoku(high, low)
```

## References

- Goichi Hosoda, Ichimoku Kinko Hyo, 1969
- Nicole Elliott, Ichimoku Charts, Harriman House, 2007
