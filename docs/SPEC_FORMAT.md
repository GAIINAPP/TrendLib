# Indicator folder format

Each indicator is one folder, `crates/trendlib/src/indicators/<name>/`:

| File | Holds | Never holds |
| --- | --- | --- |
| `spec.yaml` | Metadata: names, inputs, params, outputs, flags, TA-Lib alias mapping | Logic, conditions, formulas |
| `mod.rs` | Params struct, `validate`, `lookback`, batch, stream | Defaults, ranges, names that belong in the spec |
| `doc.md` | Summary, formula, conventions, example, references | Implementation details (guards, epsilons) |
| `golden/*.csv` | Oracle input and expected output, one file per parameter case | Values computed by TrendLib |

The generator validates every rule below and fails with `file:line` messages.

## 1. `spec.yaml`

```yaml
name: bbands                      # required; lowercase snake_case; equals the folder name
title: Bollinger Bands            # required; one line, no trailing period
group: overlap                    # required; a key in indicators/_groups.yaml
flags: [overlap]                  # optional; see § 1.1
inputs:                           # required; order = Python positional order
  - name: source
    kind: series                  # series | open | high | low | close | volume | timestamps
params:                           # optional; order = documented order (all keyword-only)
  - name: period
    type: int                     # int | float | bool | tz | enum:<EnumName>
    default: 20
    min: 2                        # inclusive; omit min/max for "any finite value"
    max: 100000
    doc: Number of bars in the average and the deviation window
  - name: nbdev_up
    type: float
    default: 2.0
    doc: Standard deviations added to the middle band
  - name: nbdev_dn
    type: float
    default: 2.0
    doc: Standard deviations subtracted from the middle band
  - name: ma_type
    type: enum:MaType
    default: sma
    doc: Moving average used for the middle band
outputs:                          # required; order = tuple order
  - name: bbands_upper
    dtype: float64                # float64 | int32
    plot: upper_band              # see § 1.2
    doc: Middle band plus nbdev_up standard deviations
  - name: bbands_middle
    dtype: float64
    plot: middle_band
    doc: Moving average of source
  - name: bbands_lower
    dtype: float64
    plot: lower_band
    doc: Middle band minus nbdev_dn standard deviations
talib:                            # optional; present only if TA-Lib has this indicator
  name: BBANDS
  params: {period: timeperiod, nbdev_up: nbdevup, nbdev_dn: nbdevdn, ma_type: matype}
  outputs: [upperband, middleband, lowerband]   # same order as outputs above
references:
  - John Bollinger, Bollinger on Bollinger Bands, McGraw-Hill, 2001
see_also: [sma, atr]
```

Rules:

- Names are unique across the catalogue: no two indicators share a function name,
  no output name repeats an input name, output names are unique within the
  indicator.
- `default` lies within `[min, max]`. Enum defaults are one of the enum's values.
- An input may carry `optional: true` (e.g. `timestamps` when `anchor="none"`).
  Whether it is required for given params is decided in `mod.rs`, not here.
- `kind: series` means "any single series"; given a DataFrame, its `close` column
  is used.
- Text fields are plain text; no Markdown, no trailing whitespace.

### 1.1 Flags

| Flag | Meaning | Effect |
| --- | --- | --- |
| `overlap` | Drawn on the price pane | Plot hint in registry/docs |
| `path_dependent` | Result depends on where computation starts | Docs note; parity tests also check slice behaviour is documented |
| `unstable` | Recursive, converges over time (EMA family) | Docs note on discarding early rows |
| `requires_timestamps` | Needs timestamps for some params | Python layer converts timestamps and offsets |
| `pattern` | Candlestick pattern, int32 ±100 output | Registry grouping, docs |
| `nan_inf_output` | A finite input may produce `NaN` or infinity | The edge-case suite checks the indicator answered, not that it answered finitely |

### 1.2 Plot hints

`line`, `histogram`, `upper_band`, `middle_band`, `lower_band`, `level`,
`direction`, `pattern`. They are hints for chart builders (GAIIN's UI uses them);
they never change computation.

## 2. Shared files

`indicators/_groups.yaml` (order = display order):

```yaml
- { key: overlap,    title: Overlap studies }
- { key: momentum,   title: Momentum }
- { key: volatility, title: Volatility }
- { key: volume,     title: Volume }
- { key: price,      title: Price transforms }
- { key: cycle,      title: Cycle }
- { key: statistic,  title: Statistics }
- { key: math,       title: Math transforms }
- { key: operator,   title: Math operators }
- { key: levels,     title: Levels }
- { key: patterns,   title: Candlestick patterns }
```

`indicators/_enums.yaml`:

```yaml
MaType:
  values: [sma, ema, wma, dema, tema]
  talib_int: { sma: 0, ema: 1, wma: 2, dema: 3, tema: 4 }
VwapAnchor:
  values: [day, none]
```

Adding an enum value is a minor change; removing or renumbering one is breaking.

## 3. `doc.md`

````markdown
# Bollinger Bands

One paragraph: what it measures and how it is commonly read. This paragraph
becomes the Python docstring summary, so keep it under 60 words and descriptive,
never advisory.

## Formula

$$
\text{middle}_t = \mathrm{MA}(x, n)_t \qquad
\sigma_t = \sqrt{\tfrac{1}{n}\textstyle\sum_{i=t-n+1}^{t} (x_i - \mathrm{SMA}(x,n)_t)^2}
$$

$$
\text{upper}_t = \text{middle}_t + k_{up}\,\sigma_t \qquad
\text{lower}_t = \text{middle}_t - k_{dn}\,\sigma_t
$$

where $x$ is `source`, $n$ is `period`.

## Conventions

- Population standard deviation (divides by n), as TA-Lib.
- Warm-up, seeding, path dependence, deviations (cite `CONVENTIONS.md` § 9 numbers),
  known TradingView differences.

## Example

```python
import trendlib as tl
upper, middle, lower = tl.bbands(close, period=20)
```

## References

- John Bollinger, *Bollinger on Bollinger Bands*, McGraw-Hill, 2001.
````

Rules: the formula is the original algebra, with no zero guards, epsilons or
`period == 1` special cases. Sections appear in this order; the generator reads
the summary paragraph and the Formula section.

## 4. Golden CSV (`golden/<case>.csv`)

One file per parameter case. `default.csv` uses default parameters and is required;
add cases for boundary parameters (minimum period at least).

```
# indicator: rsi
# case: default
# params: period=14
# oracle: ta-lib-python 0.8.1 (TA-Lib C 0.8.1), talib.RSI
# produced_by: python scripts/oracle/talib_golden.py rsi --case default
# input: testdata/daily_2000.csv (all rows)
# tolerance: rel=1e-10 abs=1e-12
# excluded_rows: none
# date: 2026-10-12
source,rsi
1003.2117466418392,nan
...
```

- Header lines start with `# key: value`. All keys shown are required.
  `excluded_rows` lists row ranges and the reason (e.g. `0-3 (Deviation 2)`) or
  `none`.
- Columns: every input (spec names, spec order), then every output (spec names).
  Timestamps are int64 epoch nanoseconds UTC; for `tz`-dependent cases the
  `params` line includes `tz=…`.
- Floats are written with round-trip precision (Python `repr`); missing is `nan`.
- Files over 2 MB are not allowed; use fewer rows.
- For human-transcribed oracles, `oracle` names the platform, symbol or input
  source, the transcriber and the date, and `produced_by` is `manual`.
