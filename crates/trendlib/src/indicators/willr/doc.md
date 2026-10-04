# Williams Percent Range

Where the latest close sits inside the range the bars have covered recently, on
a scale from -100 at the bottom of that range to 0 at the top. It is a position,
not a speed: a close pinned near the top reads near 0 whether it got there
slowly or in one bar.

## Formula

$$
\mathrm{HH}_t = \max_{i=t-n+1}^{t} h_i \qquad
\mathrm{LL}_t = \min_{i=t-n+1}^{t} l_i
$$

$$
\mathrm{WILLR}_t = -100\,\frac{\mathrm{HH}_t - c_t}{\mathrm{HH}_t - \mathrm{LL}_t}
$$

where $h$, $l$ and $c$ are `high`, `low` and `close`, and $n$ is `period`.

## Conventions

- Lookback is $n - 1$.
- When the window's high and low are equal there is no range for the close to
  sit in, and the value is reported as 0 rather than as a division by zero.
  TA-Lib answers the same.
- Not recursive: each row depends only on the last $n$ bars.
- Leading `NaN` rows are skipped; a `NaN` or infinity after the first valid bar
  raises `InvalidInput` (Deviation 1).

## Example

```python
import trendlib as tl

position = tl.willr(df, period=14)
```

## References

- Larry Williams, *How I Made One Million Dollars Last Year Trading
  Commodities*, Conceptual Management, 1973.
