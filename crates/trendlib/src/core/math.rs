//! Shared kernels.
//!
//! Each kernel is one step function used by both the batch loop and the stream,
//! so `docs/CONVENTIONS.md` § 1 (identical arithmetic in both paths) holds by
//! construction rather than by review. `push` commits a bar; `preview` returns
//! what the next `push` would return and touches nothing.

use crate::TlError;
use crate::core::hilbert::{self, Hilbert};

/// Mean of the last `period` values, kept as a running sum.
///
/// The sum is advanced the way TA-Lib advances it: add the newest value,
/// divide, then subtract the oldest. Any other order drifts by a few ULP from
/// the oracle (`docs/CONVENTIONS.md` § 1).
#[derive(Clone, Debug)]
pub struct RollingMean {
    window: Box<[f64]>,
    period: f64,
    head: usize,
    seen: usize,
    total: f64,
}

impl RollingMean {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be at least 1");
        Self {
            window: vec![0.0; period].into_boxed_slice(),
            period: period as f64,
            head: 0,
            seen: 0,
            total: 0.0,
        }
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        self.total += value;
        self.window[self.head] = value;
        self.head = (self.head + 1) % self.window.len();
        self.seen += 1;
        if self.seen < self.window.len() {
            return None;
        }
        let mean = self.total / self.period;
        self.total -= self.window[self.head];
        Some(mean)
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        if self.seen + 1 < self.window.len() {
            return None;
        }
        Some((self.total + value) / self.period)
    }
}

/// Mean of the last `period` values weighted 1, 2, ... `period`, newest heaviest.
///
/// The weighted sum is rebuilt from the window on every bar rather than
/// advanced. Advancing it is algebraically exact - adding `period * newest` and
/// subtracting the plain window sum shifts every older weight down by one - but
/// that subtraction takes two numbers of the window's own magnitude to leave a
/// difference much smaller than either, and the residue never washes out. A
/// window of 1.0 reached after a few values near 1e5 answered 1.000000001
/// instead of 1.0, ten times the tolerance this project allows. Rebuilding
/// costs one pass over the window, which is the same pass `RollingWindow`
/// spends for the same reason.
#[derive(Clone, Debug)]
pub struct WeightedMean {
    window: Box<[f64]>,
    head: usize,
    seen: usize,
    divider: f64,
}

impl WeightedMean {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be at least 1");
        let period = period as f64;
        Self {
            window: vec![0.0; period as usize].into_boxed_slice(),
            head: 0,
            seen: 0,
            divider: period * (period + 1.0) / 2.0,
        }
    }

    /// Oldest value first, so the weights run 1, 2, ... period.
    ///
    /// Returns `None` when every value in the window is the same number. Any
    /// weighted mean of identical values is that value, but computing it as a
    /// sum of multiples divided by the weight total does not always give it
    /// back, so the caller answers directly instead of through the division.
    fn weighted(&self, oldest: usize, replacing: Option<(usize, f64)>) -> Option<f64> {
        let period = self.window.len();
        let at = |offset: usize| {
            let slot = (oldest + offset) % period;
            match replacing {
                Some((replaced, with)) if replaced == slot => with,
                _ => self.window[slot],
            }
        };
        let first = at(0);
        let mut total = 0.0;
        let mut flat = true;
        for offset in 0..period {
            let value = at(offset);
            flat &= value == first;
            total += value * (offset + 1) as f64;
        }
        (!flat).then_some(total)
    }

    fn mean(&self, oldest: usize, replacing: Option<(usize, f64)>) -> f64 {
        match self.weighted(oldest, replacing) {
            Some(total) => total / self.divider,
            None => match replacing {
                Some((_, value)) => value,
                None => self.window[oldest],
            },
        }
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        let period = self.window.len();
        self.window[self.head] = value;
        self.head = (self.head + 1) % period;
        self.seen += 1;
        if self.seen < period {
            return None;
        }
        Some(self.mean(self.head, None))
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        let period = self.window.len();
        if self.seen + 1 < period {
            return None;
        }
        // The bar being offered would land in the slot the oldest occupies, and
        // the window would then start one slot further on.
        let oldest = (self.head + 1) % period;
        Some(self.mean(oldest, Some((self.head, value))))
    }
}

/// The greater of today's range and today's gap from the previous close.
///
/// Needs one earlier bar, so its lookback is 1.
#[derive(Clone, Debug, Default)]
pub struct TrueRange {
    previous_close: Option<f64>,
}

impl TrueRange {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn range(high: f64, low: f64, previous_close: f64) -> f64 {
        let span = high - low;
        let up = (high - previous_close).abs();
        let down = (low - previous_close).abs();
        span.max(up).max(down)
    }

    pub fn push(&mut self, high: f64, low: f64, close: f64) -> Option<f64> {
        let previous = self.previous_close.replace(close)?;
        Some(Self::range(high, low, previous))
    }

    pub fn preview(&self, high: f64, low: f64) -> Option<f64> {
        Some(Self::range(high, low, self.previous_close?))
    }
}

/// Exponential moving average, seeded with the simple mean of the first
/// `period` values and advanced as `prev + (x - prev) * k`.
#[derive(Clone, Debug)]
pub struct Ema {
    period: usize,
    period_f64: f64,
    k: f64,
    seen: usize,
    seed_total: f64,
    prev: f64,
}

impl Ema {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be at least 1");
        Self {
            period,
            period_f64: period as f64,
            k: 2.0 / (period as f64 + 1.0),
            seen: 0,
            seed_total: 0.0,
            prev: f64::NAN,
        }
    }

    /// An exponential average seeded over `period` bars but smoothed with a
    /// constant of its own.
    ///
    /// `macdfix` is the reason this exists: TA-Lib writes its two constants as
    /// the literals 0.075 and 0.15, which are not `2 / (26 + 1)` and
    /// `2 / (12 + 1)`, so the fixed MACD is not the 12/26 one.
    pub fn with_smoothing(period: usize, k: f64) -> Self {
        Self {
            k,
            ..Self::new(period)
        }
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        if self.seen < self.period {
            self.seed_total += value;
            self.seen += 1;
            if self.seen < self.period {
                return None;
            }
            self.prev = self.seed_total / self.period_f64;
            return Some(self.prev);
        }
        self.prev = self.step(value);
        self.seen += 1;
        Some(self.prev)
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        match (self.seen + 1).cmp(&self.period) {
            std::cmp::Ordering::Less => None,
            std::cmp::Ordering::Equal => Some((self.seed_total + value) / self.period_f64),
            std::cmp::Ordering::Greater => Some(self.step(value)),
        }
    }

    fn step(&self, value: f64) -> f64 {
        // A period of 1 weights the new bar fully, so the average is the series
        // itself. Taking the general path there would subtract two values of
        // very different magnitude and add the difference back, losing the
        // smaller one: EMA(1) of [16384.0, -9.7e-11] would return -9.8e-11
        // instead of the bar that was just handed in.
        if self.period == 1 {
            return value;
        }
        ((value - self.prev) * self.k) + self.prev
    }
}

/// The highest or lowest value in the last `period` bars.
///
/// The extreme is tracked rather than rescanned: a new bar only has to beat the
/// one on record, and the window is rescanned only on the bar where the record
/// holder drops out of it. That keeps a batch run linear in the common case
/// instead of multiplying bars by period.
#[derive(Clone, Debug)]
pub struct RollingExtreme {
    window: Box<[f64]>,
    /// Which bar wins when two tie. TA-Lib is not consistent about this, so
    /// each consumer says what it needs: `maxindex` keeps the earliest tying
    /// bar and walks its index forward only as the record leaves the window,
    /// while `aroon` keeps the latest and reads 100 across a flat stretch.
    prefer_recent: bool,
    /// The absolute row each slot was written from, so the extreme can say
    /// where it came from and not merely what it was.
    rows: Box<[u64]>,
    head: usize,
    seen: usize,
    best: usize,
    want_max: bool,
}

impl RollingExtreme {
    pub fn highest(period: usize) -> Self {
        Self::new(period, true)
    }

    pub fn lowest(period: usize) -> Self {
        Self::new(period, false)
    }

    fn new(period: usize, want_max: bool) -> Self {
        assert!(period > 0, "period must be at least 1");
        Self {
            window: vec![0.0; period].into_boxed_slice(),
            prefer_recent: false,
            rows: vec![0; period].into_boxed_slice(),
            head: 0,
            seen: 0,
            best: 0,
            want_max,
        }
    }

    /// Keep the most recent of two tying bars instead of the earliest.
    pub fn preferring_recent(mut self) -> Self {
        self.prefer_recent = true;
        self
    }

    fn beats(&self, candidate: f64, incumbent: f64) -> bool {
        if self.want_max {
            candidate > incumbent
        } else {
            candidate < incumbent
        }
    }

    /// Whether an arriving bar displaces the record, ties included or not
    /// according to this kernel's preference.
    fn displaces(&self, candidate: f64, incumbent: f64) -> bool {
        self.beats(candidate, incumbent) || (self.prefer_recent && candidate == incumbent)
    }

    /// The best of the written slots, ignoring one. `None` when every slot was
    /// ignored, which happens with `period = 1`: the bar being offered is then
    /// the only candidate there is.
    fn scan(&self, filled: usize, skip: Option<usize>) -> Option<usize> {
        let mut best: Option<usize> = None;
        for slot in 0..filled {
            if Some(slot) == skip {
                continue;
            }
            let better = match best {
                None => true,
                Some(current) => {
                    self.beats(self.window[slot], self.window[current])
                        // Slots are not scanned in time order, so a tie has
                        // to be settled by comparing the rows themselves.
                        || (self.window[slot] == self.window[current]
                            && if self.prefer_recent {
                                self.rows[slot] > self.rows[current]
                            } else {
                                self.rows[slot] < self.rows[current]
                            })
                }
            };
            if better {
                best = Some(slot);
            }
        }
        best
    }

    /// Bars given to this kernel so far.
    pub fn seen(&self) -> usize {
        self.seen
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        let period = self.window.len();
        let slot = self.head;
        let dropped_record = self.seen >= period && slot == self.best;

        self.window[slot] = value;
        self.rows[slot] = self.seen as u64;
        self.head = (self.head + 1) % period;
        self.seen += 1;

        if dropped_record {
            self.best = self.scan(period, None).expect("a full window has a best");
        } else if self.seen == 1 || self.displaces(value, self.window[self.best]) {
            self.best = slot;
        }

        (self.seen >= period).then(|| self.window[self.best])
    }

    /// The absolute row the current extreme came from, counting from zero at
    /// the first bar the kernel was given.
    pub fn best_row(&self) -> Option<u64> {
        (self.seen >= self.window.len()).then(|| self.rows[self.best])
    }

    /// How many bars ago the current extreme was set.
    pub fn bars_since_best(&self) -> Option<u64> {
        self.best_row().map(|row| self.seen as u64 - 1 - row)
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        self.preview_best(value).map(|(value, _)| value)
    }

    /// How many bars ago the extreme would be after pushing `value`.
    pub fn preview_bars_since_best(&self, value: f64) -> Option<u64> {
        self.preview_best(value)
            .map(|(_, row)| self.seen as u64 - row)
    }

    /// What the extreme and the row it came from would be after pushing
    /// `value`, without pushing it.
    fn preview_best(&self, value: f64) -> Option<(f64, u64)> {
        let period = self.window.len();
        if self.seen + 1 < period {
            return None;
        }
        let arriving = self.seen as u64;
        let incumbent = if self.seen < period {
            // The window fills exactly on this bar, so nothing drops out.
            self.scan(self.seen, None)
        } else if self.head == self.best {
            self.scan(period, Some(self.head))
        } else {
            Some(self.best)
        };
        let Some(slot) = incumbent else {
            return Some((value, arriving));
        };
        if self.displaces(value, self.window[slot]) {
            Some((value, arriving))
        } else {
            Some((self.window[slot], self.rows[slot]))
        }
    }
}

/// The last `period` values, for indicators that need the whole window rather
/// than a running summary.
#[derive(Clone, Debug)]
pub struct RollingWindow {
    window: Box<[f64]>,
    head: usize,
    seen: usize,
    total: f64,
}

impl RollingWindow {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be at least 1");
        Self {
            window: vec![0.0; period].into_boxed_slice(),
            head: 0,
            seen: 0,
            total: 0.0,
        }
    }

    pub fn period(&self) -> usize {
        self.window.len()
    }

    pub fn is_full(&self) -> bool {
        self.seen >= self.window.len()
    }

    pub fn push(&mut self, value: f64) -> bool {
        if self.is_full() {
            self.total -= self.window[self.head];
        }
        self.total += value;
        self.window[self.head] = value;
        self.head = (self.head + 1) % self.window.len();
        self.seen += 1;
        self.is_full()
    }

    /// The running mean. Cheap, but it carries the drift of every value ever
    /// added; use [`Self::exact_mean`] where a later subtraction will magnify
    /// that drift.
    pub fn mean(&self) -> f64 {
        self.total / self.window.len() as f64
    }

    /// The mean re-summed from the window, oldest value first.
    ///
    /// An indicator that subtracts this mean from a value of the same size
    /// loses most of the significant digits in the subtraction, so a running
    /// total's accumulated drift turns into a visible error. Re-summing costs
    /// one pass over the window and removes it.
    pub fn exact_mean(&self) -> f64 {
        self.values().sum::<f64>() / self.window.len() as f64
    }

    /// The window oldest value first. When full, `head` is the oldest slot.
    pub fn values(&self) -> impl Iterator<Item = f64> + '_ {
        let period = self.window.len();
        let start = if self.is_full() { self.head } else { 0 };
        (0..period).map(move |offset| self.window[(start + offset) % period])
    }

    /// The window as it would be after pushing `value`, without pushing it.
    pub fn preview_values(&self, value: f64) -> impl Iterator<Item = f64> + '_ {
        let period = self.window.len();
        let dropping = self.head;
        let start = if self.preview_is_full() && self.is_full() {
            (self.head + 1) % period
        } else {
            0
        };
        (0..period).map(move |offset| {
            let slot = (start + offset) % period;
            if slot == dropping {
                value
            } else {
                self.window[slot]
            }
        })
    }

    pub fn preview_mean(&self, value: f64) -> f64 {
        self.preview_values(value).sum::<f64>() / self.window.len() as f64
    }

    pub fn preview_is_full(&self) -> bool {
        self.seen + 1 >= self.window.len()
    }
}

/// Wilder's running total: the plain sum of the first `period` values, then
/// `prev - prev / n + x`.
///
/// This is the form the directional movement family and the Chande momentum
/// oscillator accumulate with. It is `period` times [`Wilder`], which keeps the
/// same quantity as an average; both exist because TA-Lib reports some
/// indicators from one and some from the other, and scaling between them after
/// the fact would not round identically.
#[derive(Clone, Debug)]
pub struct WilderSum {
    period: usize,
    n: f64,
    seen: usize,
    total: f64,
}

impl WilderSum {
    pub fn new(period: usize) -> Self {
        Self::with_seed(period, period)
    }

    /// A total whose seed is a different length from its divisor.
    ///
    /// The directional movement family needs this: TA-Lib seeds `+DM` and
    /// `-DM` with `period - 1` movements and then decays by `period`, which is
    /// why their lookback is one bar shorter than the accumulation suggests.
    pub fn with_seed(seed: usize, divisor: usize) -> Self {
        assert!(seed > 0, "seed must be at least 1");
        assert!(divisor > 0, "divisor must be at least 1");
        Self {
            period: seed,
            n: divisor as f64,
            seen: 0,
            total: 0.0,
        }
    }

    fn advance(total: f64, n: f64, value: f64) -> f64 {
        total - total / n + value
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        if self.seen < self.period {
            self.total += value;
            self.seen += 1;
            return (self.seen == self.period).then_some(self.total);
        }
        self.total = Self::advance(self.total, self.n, value);
        self.seen += 1;
        Some(self.total)
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        match (self.seen + 1).cmp(&self.period) {
            std::cmp::Ordering::Less => None,
            std::cmp::Ordering::Equal => Some(self.total + value),
            std::cmp::Ordering::Greater => Some(Self::advance(self.total, self.n, value)),
        }
    }
}

/// Wilder's directional indicators, `+DI` and `-DI`.
///
/// Both are the running total of movement in one direction as a percentage of
/// the running total of true range, so they answer "how much of the ground
/// covered was in this direction". The movement and the range totals are both
/// seeded over `period - 1` bars and decayed by `period`, but the ratio is not
/// reported on the bar they first exist: TA-Lib starts a bar later, which is
/// why the lookback is `period` rather than `period - 1`.
#[derive(Clone, Debug)]
pub struct Directional {
    previous: Option<(f64, f64, f64)>,
    plus: WilderSum,
    minus: WilderSum,
    range: WilderSum,
    seen: usize,
    period: usize,
}

impl Directional {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be at least 1");
        let seed = period.saturating_sub(1).max(1);
        Self {
            previous: None,
            plus: WilderSum::with_seed(seed, period),
            minus: WilderSum::with_seed(seed, period),
            range: WilderSum::with_seed(seed, period),
            seen: 0,
            period,
        }
    }

    /// The movement a bar contributes in each direction. Only the larger of the
    /// two edge extensions counts, and only if it extended at all, so a bar
    /// inside the previous one contributes to neither.
    fn movement(previous: (f64, f64, f64), high: f64, low: f64) -> (f64, f64) {
        let up = high - previous.0;
        let down = previous.1 - low;
        let plus = if up > down && up > 0.0 { up } else { 0.0 };
        let minus = if down > up && down > 0.0 { down } else { 0.0 };
        (plus, minus)
    }

    fn indicators(plus: f64, minus: f64, range: f64) -> (f64, f64) {
        if range == 0.0 {
            (0.0, 0.0)
        } else {
            (100.0 * plus / range, 100.0 * minus / range)
        }
    }

    pub fn push(&mut self, high: f64, low: f64, close: f64) -> Option<(f64, f64)> {
        let previous = self.previous.replace((high, low, close))?;
        let (up, down) = Self::movement(previous, high, low);
        let range = TrueRange::range(high, low, previous.2);
        let plus = self.plus.push(up);
        let minus = self.minus.push(down);
        let range = self.range.push(range);
        self.seen += 1;
        // The totals exist a bar before the ratio is reported.
        if self.seen < self.period {
            return None;
        }
        Some(Self::indicators(plus?, minus?, range?))
    }

    /// `close` is taken for symmetry with `push`; the range this bar adds is
    /// measured against the *previous* close, which is already held.
    pub fn preview(&self, high: f64, low: f64, _close: f64) -> Option<(f64, f64)> {
        let previous = self.previous?;
        if self.seen + 1 < self.period {
            return None;
        }
        let (up, down) = Self::movement(previous, high, low);
        let range = TrueRange::range(high, low, previous.2);
        Some(Self::indicators(
            self.plus.preview(up)?,
            self.minus.preview(down)?,
            self.range.preview(range)?,
        ))
    }
}

/// The spread between the two directional indicators, as a share of their sum.
pub fn directional_index(plus: f64, minus: f64) -> f64 {
    let total = plus + minus;
    if total == 0.0 {
        0.0
    } else {
        100.0 * (plus - minus).abs() / total
    }
}

/// The value `lag` bars ago, for indicators that compare now with then.
#[derive(Clone, Debug)]
pub struct Lagged {
    window: Box<[f64]>,
    head: usize,
    seen: usize,
}

impl Lagged {
    pub fn new(lag: usize) -> Self {
        assert!(lag > 0, "lag must be at least 1");
        Self {
            window: vec![0.0; lag].into_boxed_slice(),
            head: 0,
            seen: 0,
        }
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        let past = self.earlier();
        self.window[self.head] = value;
        self.head = (self.head + 1) % self.window.len();
        self.seen += 1;
        past
    }

    /// The value that a `push` now would compare against. Unaffected by the
    /// bar being offered, so `preview` can use it directly.
    pub fn earlier(&self) -> Option<f64> {
        (self.seen >= self.window.len()).then(|| self.window[self.head])
    }
}

/// Wilder's smoothing: the simple average of the first `period` values, then
/// `prev * (n - 1) / n + x / n` (`docs/CONVENTIONS.md` § 4).
#[derive(Clone, Debug)]
pub struct Wilder {
    period: usize,
    n: f64,
    n_minus_one: f64,
    seen: usize,
    seed_total: f64,
    prev: f64,
}

impl Wilder {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be at least 1");
        Self {
            period,
            n: period as f64,
            n_minus_one: (period - 1) as f64,
            seen: 0,
            seed_total: 0.0,
            prev: f64::NAN,
        }
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        if self.seen < self.period {
            self.seed_total += value;
            self.seen += 1;
            if self.seen < self.period {
                return None;
            }
            self.prev = self.seed_total / self.n;
            return Some(self.prev);
        }
        self.prev = self.prev * self.n_minus_one / self.n + value / self.n;
        self.seen += 1;
        Some(self.prev)
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        match (self.seen + 1).cmp(&self.period) {
            std::cmp::Ordering::Less => None,
            std::cmp::Ordering::Equal => Some((self.seed_total + value) / self.n),
            std::cmp::Ordering::Greater => {
                Some(self.prev * self.n_minus_one / self.n + value / self.n)
            }
        }
    }
}

/// An exponential average seeded with the first value it is given rather than
/// with the mean of the first `period`.
///
/// The Chaikin oscillator needs it: its two averages run over a cumulative
/// line that starts at the first bar and has no warm-up of its own to average.
#[derive(Clone, Debug)]
pub struct EmaFromFirst {
    k: f64,
    previous: Option<f64>,
}

impl EmaFromFirst {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be at least 1");
        Self {
            k: 2.0 / (period as f64 + 1.0),
            previous: None,
        }
    }

    pub fn push(&mut self, value: f64) -> f64 {
        let next = self.preview(value);
        self.previous = Some(next);
        next
    }

    pub fn preview(&self, value: f64) -> f64 {
        match self.previous {
            Some(previous) => (value - previous) * self.k + previous,
            None => value,
        }
    }
}

/// Two moving averages of one series, the shorter period first.
///
/// TA-Lib sorts the two periods before it starts, so asking for a fast period
/// longer than the slow one gives the same answer as asking for them the other
/// way round, not the negated one.
#[derive(Clone, Debug)]
pub struct MaPair {
    fast: MovingAverage,
    slow: MovingAverage,
}

impl MaPair {
    pub fn new(kind: MaType, fast_period: usize, slow_period: usize) -> Self {
        let (fast, slow) = Self::sorted(fast_period, slow_period);
        Self {
            fast: kind.state(fast),
            slow: kind.state(slow),
        }
    }

    pub fn sorted(fast: usize, slow: usize) -> (usize, usize) {
        if fast > slow {
            (slow, fast)
        } else {
            (fast, slow)
        }
    }

    pub fn lookback(kind: MaType, fast: usize, slow: usize) -> usize {
        kind.lookback(Self::sorted(fast, slow).1)
    }

    pub fn push(&mut self, value: f64) -> Option<(f64, f64)> {
        // The shorter average is ready first and has to keep being fed while
        // the longer one warms up; returning early here would starve it.
        let fast = self.fast.push(value);
        let slow = self.slow.push(value)?;
        Some((fast?, slow))
    }

    pub fn preview(&self, value: f64) -> Option<(f64, f64)> {
        Some((self.fast.preview(value)?, self.slow.preview(value)?))
    }
}

/// Where the close sits inside the high-low range of the last `period` bars,
/// as a percentage: the raw %K the stochastic family is built on.
#[derive(Clone, Debug)]
pub struct Stochastic {
    highest: RollingExtreme,
    lowest: RollingExtreme,
}

impl Stochastic {
    pub fn new(period: usize) -> Self {
        Self {
            highest: RollingExtreme::highest(period),
            lowest: RollingExtreme::lowest(period),
        }
    }

    pub fn push(&mut self, high: f64, low: f64, close: f64) -> Option<f64> {
        // Both windows have to take the bar; returning on the first would
        // leave the other one short.
        let top = self.highest.push(high);
        let bottom = self.lowest.push(low)?;
        Some(Self::percent(close, top?, bottom))
    }

    pub fn preview(&self, high: f64, low: f64, close: f64) -> Option<f64> {
        let top = self.highest.preview(high)?;
        let bottom = self.lowest.preview(low)?;
        Some(Self::percent(close, top, bottom))
    }

    /// A window with no range has nowhere for the close to sit, and TA-Lib
    /// answers zero rather than dividing. The test is exact: a range that is
    /// merely small still places the close somewhere.
    fn percent(close: f64, top: f64, bottom: f64) -> f64 {
        let range = top - bottom;
        if range == 0.0 {
            0.0
        } else {
            100.0 * ((close - bottom) / range)
        }
    }
}

/// Two series over the same window, for the statistics that compare them.
///
/// The sums are re-added from the windows each bar rather than carried
/// forward, because both `beta` and `correl` subtract quantities of the same
/// size and a running total's drift would survive the subtraction.
#[derive(Clone, Debug)]
pub struct Paired {
    first: RollingWindow,
    second: RollingWindow,
}

/// How far each series spread about its own mean over the window, and how much
/// of that spread they shared.
#[derive(Clone, Copy, Debug)]
pub struct Sums {
    pub count: f64,
    /// The sum of squared deviations of each series from its own mean.
    pub first_spread: f64,
    pub second_spread: f64,
    /// The sum of the products of the two deviations.
    pub shared: f64,
}

impl Paired {
    pub fn new(period: usize) -> Self {
        Self {
            first: RollingWindow::new(period),
            second: RollingWindow::new(period),
        }
    }

    pub fn push(&mut self, first: f64, second: f64) -> Option<Sums> {
        let full = self.first.push(first);
        self.second.push(second);
        full.then(|| {
            Self::sums(
                self.first.values(),
                self.second.values(),
                self.first.period(),
            )
        })
    }

    pub fn preview(&self, first: f64, second: f64) -> Option<Sums> {
        self.first.preview_is_full().then(|| {
            Self::sums(
                self.first.preview_values(first),
                self.second.preview_values(second),
                self.first.period(),
            )
        })
    }

    /// Two passes: the means, then the deviations from them.
    ///
    /// Subtracting the square of a mean from the mean of the squares is one
    /// pass cheaper and loses most of the digits where the two series move
    /// together, which is exactly where these statistics are used. Over two
    /// bars the correlation is `±1` and this reaches it exactly on 1384 of the
    /// dataset's rows where the one-pass form reaches it on 981.
    fn sums(
        first: impl Iterator<Item = f64>,
        second: impl Iterator<Item = f64>,
        period: usize,
    ) -> Sums {
        let count = period as f64;
        let first: Vec<f64> = first.collect();
        let second: Vec<f64> = second.collect();
        let first_mean = first.iter().sum::<f64>() / count;
        let second_mean = second.iter().sum::<f64>() / count;
        let mut totals = Sums {
            count,
            first_spread: 0.0,
            second_spread: 0.0,
            shared: 0.0,
        };
        for (x, y) in first.into_iter().zip(second) {
            let dx = x - first_mean;
            let dy = y - second_mean;
            totals.first_spread += dx * dx;
            totals.second_spread += dy * dy;
            totals.shared += dx * dy;
        }
        totals
    }
}

/// A least squares line fitted to the last `period` values.
///
/// The x axis runs backwards, `period - 1` at the oldest bar down to 0 at the
/// newest, and the divisor is negated to match. That is how TA-Lib arranges
/// it; the two signs cancel, so the slope is the ordinary one.
#[derive(Clone, Debug)]
pub struct LinearRegression {
    window: RollingWindow,
    sum_x: f64,
    divisor: f64,
}

/// The line fitted over one window.
#[derive(Clone, Copy, Debug)]
pub struct Fit {
    pub slope: f64,
    pub intercept: f64,
    period: f64,
}

impl Fit {
    /// The fitted value at the newest bar in the window.
    pub fn at_last_bar(self) -> f64 {
        self.intercept + self.slope * (self.period - 1.0)
    }

    /// The line carried one bar past the newest.
    pub fn one_bar_ahead(self) -> f64 {
        self.intercept + self.slope * self.period
    }

    /// The slope as an angle in degrees.
    pub fn angle(self) -> f64 {
        self.slope.atan() * (180.0 / std::f64::consts::PI)
    }
}

impl LinearRegression {
    pub fn new(period: usize) -> Self {
        let n = period as u64;
        let sum_x = (n * (n - 1)) as f64 * 0.5;
        // Always divisible by 6, so integer division loses nothing.
        let sum_xx = (n * (n - 1) * (2 * n - 1) / 6) as f64;
        Self {
            window: RollingWindow::new(period),
            sum_x,
            divisor: sum_x * sum_x - period as f64 * sum_xx,
        }
    }

    pub fn push(&mut self, value: f64) -> Option<Fit> {
        if !self.window.push(value) {
            return None;
        }
        Some(self.fit(self.window.values()))
    }

    pub fn preview(&self, value: f64) -> Option<Fit> {
        if !self.window.preview_is_full() {
            return None;
        }
        Some(self.fit(self.window.preview_values(value)))
    }

    fn fit(&self, values: impl Iterator<Item = f64>) -> Fit {
        let period = self.window.period();
        let n = period as f64;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        for (offset, value) in values.enumerate() {
            sum_y += value;
            sum_xy += (period - 1 - offset) as f64 * value;
        }
        let slope = (n * sum_xy - self.sum_x * sum_y) / self.divisor;
        Fit {
            slope,
            intercept: (sum_y - slope * self.sum_x) / n,
            period: n,
        }
    }
}

/// Ehlers' MESA adaptive average: its step follows how fast the dominant
/// cycle's phase is turning, so it hurries when the cycle stalls and slows
/// when it is moving.
#[derive(Clone, Debug)]
pub struct Mesa {
    cycle: Hilbert,
    fast_limit: f64,
    slow_limit: f64,
    previous_degrees: f64,
    mama: f64,
    fama: f64,
    seen: usize,
}

impl Mesa {
    /// The limits TA-Lib fixes when MESA is reached through the moving average
    /// dispatch rather than called directly.
    pub const FAST_LIMIT_DEFAULT: f64 = 0.5;
    pub const SLOW_LIMIT_DEFAULT: f64 = 0.05;
    pub const LOOKBACK: usize = 32;

    pub fn new(fast_limit: f64, slow_limit: f64) -> Self {
        Self {
            cycle: Hilbert::new(hilbert::PRIMED_EARLY),
            fast_limit,
            slow_limit,
            previous_degrees: 0.0,
            mama: 0.0,
            fama: 0.0,
            seen: 0,
        }
    }

    /// How much of the new bar the average takes. A phase that barely moved
    /// means the cycle has stalled and the average steps at its fastest; one
    /// turning quickly slows it down in proportion.
    fn step(&self, turned: f64) -> f64 {
        if turned <= 1.0 {
            self.fast_limit
        } else {
            (self.fast_limit / turned).max(self.slow_limit)
        }
    }

    /// Both lines, or `None` until the cycle reading has settled.
    pub fn push(&mut self, value: f64) -> Option<(f64, f64)> {
        let reading = self.cycle.push(value);
        self.seen += 1;
        let reading = reading?;
        let degrees = if reading.in_phase != 0.0 {
            (reading.quadrature / reading.in_phase).atan() * (180.0 / std::f64::consts::PI)
        } else {
            0.0
        };
        let turned = (self.previous_degrees - degrees).max(1.0);
        self.previous_degrees = degrees;
        let alpha = self.step(turned);
        self.mama = (1.0 - alpha).mul_add(self.mama, alpha * value);
        let half = alpha * 0.5;
        self.fama = (1.0 - half).mul_add(self.fama, half * self.mama);
        (self.seen > Self::LOOKBACK).then_some((self.mama, self.fama))
    }

    pub fn preview(&self, value: f64) -> Option<(f64, f64)> {
        let mut forked = self.clone();
        forked.push(value)
    }
}

/// Kaufman's adaptive average: smooths hard when the series travels a long way
/// to go nowhere and barely at all when it goes straight there.
#[derive(Clone, Debug)]
pub enum Adaptive {
    /// Over one bar the efficiency ratio is always 1, which would smooth at a
    /// fixed rate rather than adapt. TA-Lib returns the series itself instead.
    Passthrough,
    Measured(Measured),
}

#[derive(Clone, Debug)]
pub struct Measured {
    period: usize,
    seen: usize,
    /// The bar `period` places back, which the net change is measured against.
    anchor: Lagged,
    /// The last `period` absolute bar-to-bar moves; their sum is the distance
    /// actually travelled.
    moves: RollingWindow,
    previous: Option<f64>,
    average: Option<f64>,
}

impl Adaptive {
    /// The two ends of the smoothing range, which TA-Lib fixes at the
    /// constants for periods 2 and 30 whatever `period` is.
    pub const FASTEST: f64 = 2.0 / (2.0 + 1.0);
    pub const SLOWEST: f64 = 2.0 / (30.0 + 1.0);

    pub fn new(period: usize) -> Self {
        if period == 1 {
            return Self::Passthrough;
        }
        Self::Measured(Measured {
            period,
            seen: 0,
            anchor: Lagged::new(period),
            moves: RollingWindow::new(period),
            previous: None,
            average: None,
        })
    }

    pub fn lookback(period: usize) -> usize {
        if period == 1 { 0 } else { period }
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        let Self::Measured(state) = self else {
            return Some(value);
        };
        let anchor = state.anchor.push(value);
        if let Some(previous) = state.previous.replace(value) {
            state.moves.push((value - previous).abs());
        }
        state.seen += 1;
        if state.seen == state.period {
            // The average starts from the bar before its first value rather
            // than from an average of the window.
            state.average = Some(value);
            return None;
        }
        let next = state.next(value, anchor?, state.average?, state.moves.values().sum());
        state.average = Some(next);
        Some(next)
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        let Self::Measured(state) = self else {
            return Some(value);
        };
        let moved = (value - state.previous?).abs();
        if state.seen + 1 == state.period {
            return None;
        }
        Some(state.next(
            value,
            state.anchor.earlier()?,
            state.average?,
            state.moves.preview_values(moved).sum(),
        ))
    }
}

impl Measured {
    fn next(&self, value: f64, anchor: f64, previous: f64, travelled: f64) -> f64 {
        // The ratio cannot exceed 1, since a window cannot end further from
        // where it began than the distance it covered. Rounding can put it a
        // hair over, and a window that did not move at all puts it at 0/0;
        // both are read as fully efficient, which is the fastest smoothing.
        let change = (value - anchor).abs();
        let efficiency = if travelled <= change {
            1.0
        } else {
            change / travelled
        };
        let blend = efficiency * (Adaptive::FASTEST - Adaptive::SLOWEST) + Adaptive::SLOWEST;
        previous + blend * blend * (value - previous)
    }
}

/// Tillson's T3: six exponential stages combined so that most of the lag the
/// chaining introduced comes back out.
#[derive(Clone, Debug)]
pub struct Tillson {
    stages: [Ema; 6],
    weights: [f64; 4],
}

impl Tillson {
    /// The `v_factor` the moving-average dispatch uses, which TA-Lib fixes
    /// when T3 is reached through `MA` rather than called directly.
    pub const V_FACTOR_DEFAULT: f64 = 0.7;

    pub fn new(period: usize, v_factor: f64) -> Self {
        let v2 = v_factor * v_factor;
        let v3 = v2 * v_factor;
        Self {
            stages: std::array::from_fn(|_| Ema::new(period)),
            // Third, fourth, fifth, sixth. They sum to one at every
            // `v_factor`, so a series the stages leave unchanged comes through
            // unchanged.
            weights: [
                1.0 + 3.0 * v_factor + v3 + 3.0 * v2,
                -6.0 * v2 - 3.0 * v_factor - 3.0 * v3,
                3.0 * v2 + 3.0 * v3,
                -v3,
            ],
        }
    }

    pub fn lookback(period: usize) -> usize {
        6 * period.saturating_sub(1)
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        let mut carried = value;
        let mut reached = [0.0; 6];
        for (stage, slot) in self.stages.iter_mut().zip(&mut reached) {
            carried = stage.push(carried)?;
            *slot = carried;
        }
        Some(self.combine(reached))
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        let mut carried = value;
        let mut reached = [0.0; 6];
        for (stage, slot) in self.stages.iter().zip(&mut reached) {
            carried = stage.preview(carried)?;
            *slot = carried;
        }
        Some(self.combine(reached))
    }

    fn combine(&self, reached: [f64; 6]) -> f64 {
        self.weights[3] * reached[5]
            + self.weights[2] * reached[4]
            + self.weights[1] * reached[3]
            + self.weights[0] * reached[2]
    }
}

/// Hull's average: a half-length weighted mean doubled and the full-length one
/// taken off it, smoothed over the square root of the period.
#[derive(Clone, Debug)]
pub enum Hull {
    /// Over one bar the half-length stage would have no bars to average.
    Passthrough,
    Stages {
        half: WeightedMean,
        full: WeightedMean,
        smoothed: WeightedMean,
    },
}

impl Hull {
    fn smoothing_period(period: usize) -> usize {
        (period as f64).sqrt() as usize
    }

    pub fn new(period: usize) -> Self {
        if period == 1 {
            return Self::Passthrough;
        }
        Self::Stages {
            half: WeightedMean::new(period / 2),
            full: WeightedMean::new(period),
            smoothed: WeightedMean::new(Self::smoothing_period(period)),
        }
    }

    pub fn lookback(period: usize) -> usize {
        period.saturating_sub(1) + Self::smoothing_period(period).saturating_sub(1)
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        let Self::Stages {
            half,
            full,
            smoothed,
        } = self
        else {
            return Some(value);
        };
        // The half-length stage is ready first and has to keep taking bars
        // while the longer one warms up.
        let short = half.push(value);
        let long = full.push(value)?;
        smoothed.push(2.0 * short? - long)
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        let Self::Stages {
            half,
            full,
            smoothed,
        } = self
        else {
            return Some(value);
        };
        let short = half.preview(value)?;
        let long = full.preview(value)?;
        smoothed.preview(2.0 * short - long)
    }
}

/// An exponential average fed a series pushed forward by about the lag the
/// average will add.
#[derive(Clone, Debug)]
pub struct ZeroLag {
    /// Absent at periods 1 and 2, where half the lag rounds down to nothing
    /// and the correction has no bar to reach back to.
    earlier: Option<Lagged>,
    average: Ema,
}

impl ZeroLag {
    fn lag(period: usize) -> usize {
        period.saturating_sub(1) / 2
    }

    pub fn new(period: usize) -> Self {
        let lag = Self::lag(period);
        Self {
            earlier: (lag > 0).then(|| Lagged::new(lag)),
            average: Ema::new(period),
        }
    }

    pub fn lookback(period: usize) -> usize {
        Self::lag(period) + period.saturating_sub(1)
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        let corrected = match &mut self.earlier {
            Some(earlier) => 2.0 * value - earlier.push(value)?,
            None => value,
        };
        self.average.push(corrected)
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        let corrected = match &self.earlier {
            Some(earlier) => 2.0 * value - earlier.earlier()?,
            None => value,
        };
        self.average.preview(corrected)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Ema, Lagged, MaType, RollingExtreme, RollingMean, RollingWindow, TrueRange, WeightedMean,
        Wilder, WilderSum,
    };

    fn drive<F>(len: usize, mut step: F) -> Vec<Option<f64>>
    where
        F: FnMut(f64) -> Option<f64>,
    {
        (0..len).map(|i| step(i as f64 + 1.0)).collect()
    }

    #[test]
    fn rolling_mean_warms_up_then_slides() {
        let mut mean = RollingMean::new(3);
        let got = drive(5, |x| mean.push(x));
        assert_eq!(got, vec![None, None, Some(2.0), Some(3.0), Some(4.0)]);
    }

    #[test]
    fn rolling_mean_preview_equals_the_next_push() {
        let mut mean = RollingMean::new(4);
        for x in [1.0, 2.0, 3.0, 4.0, 5.0] {
            let previewed = mean.preview(9.5);
            let mut forked = mean.clone();
            assert_eq!(previewed, forked.push(9.5));
            mean.push(x);
        }
    }

    #[test]
    fn rolling_extreme_matches_a_brute_force_scan() {
        let mut rng: u64 = 0x5eed;
        let mut next = || {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((rng >> 33) as f64 / (1u64 << 31) as f64) * 200.0 - 100.0
        };
        let series: Vec<f64> = (0..400).map(|_| next()).collect();

        for period in [1usize, 2, 3, 7, 20] {
            let mut highest = RollingExtreme::highest(period);
            let mut lowest = RollingExtreme::lowest(period);
            for (row, &value) in series.iter().enumerate() {
                let previewed_high = highest.preview(value);
                let previewed_low = lowest.preview(value);
                let got_high = highest.push(value);
                let got_low = lowest.push(value);
                assert_eq!(
                    previewed_high, got_high,
                    "high preview, period {period}, row {row}"
                );
                assert_eq!(
                    previewed_low, got_low,
                    "low preview, period {period}, row {row}"
                );

                if row + 1 >= period {
                    let window = &series[row + 1 - period..=row];
                    let want_high = window.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    let want_low = window.iter().copied().fold(f64::INFINITY, f64::min);
                    assert_eq!(got_high, Some(want_high), "period {period}, row {row}");
                    assert_eq!(got_low, Some(want_low), "period {period}, row {row}");
                } else {
                    assert_eq!(got_high, None);
                    assert_eq!(got_low, None);
                }
            }
        }
    }

    #[test]
    fn rolling_extreme_settles_a_tie_the_way_it_was_asked_to() {
        // Keeping the earliest: the record is the oldest bar still in the
        // window, so its row crawls forward only as the window slides.
        let mut earliest = RollingExtreme::highest(3);
        for row in 0..6u64 {
            earliest.push(5.0);
            if row >= 2 {
                assert_eq!(earliest.best_row(), Some(row - 2), "row {row}");
            }
        }

        // Keeping the latest: the record is always this bar.
        let mut latest = RollingExtreme::highest(3).preferring_recent();
        for row in 0..6u64 {
            latest.push(5.0);
            if row >= 2 {
                assert_eq!(latest.best_row(), Some(row), "row {row}");
                assert_eq!(latest.bars_since_best(), Some(0));
            }
        }
    }

    #[test]
    fn rolling_extreme_reports_where_its_record_came_from() {
        let mut highest = RollingExtreme::highest(3);
        assert_eq!(highest.push(5.0), None);
        assert_eq!(highest.push(9.0), None);

        // Window rows 0..2, the record is the 9.0 written at row 1.
        assert_eq!(highest.push(3.0), Some(9.0));
        assert_eq!(highest.best_row(), Some(1));
        assert_eq!(highest.bars_since_best(), Some(1));

        // Rows 1..3, still the 9.0, now two bars back.
        assert_eq!(highest.push(4.0), Some(9.0));
        assert_eq!(highest.bars_since_best(), Some(2));

        // Rows 2..4: the 9.0 has left, so the 4.0 at row 3 takes over.
        assert_eq!(highest.push(2.0), Some(4.0));
        assert_eq!(highest.best_row(), Some(3));
        assert_eq!(highest.bars_since_best(), Some(1));
    }

    #[test]
    fn rolling_window_tracks_its_mean_and_contents() {
        let mut window = RollingWindow::new(3);
        assert!(!window.push(1.0));
        assert!(!window.push(2.0));
        assert!(window.push(3.0));
        assert_eq!(window.mean(), 2.0);
        assert_eq!(window.preview_mean(4.0), 3.0);
        let previewed: Vec<f64> = window.preview_values(4.0).collect();
        assert_eq!(previewed.iter().copied().fold(0.0, f64::max), 4.0);
        window.push(4.0);
        assert_eq!(window.mean(), 3.0);
    }

    #[test]
    fn wilder_sum_seeds_with_the_plain_total_then_decays_it() {
        let mut sum = WilderSum::new(3);
        assert_eq!(drive(3, |x| sum.push(x)), vec![None, None, Some(6.0)]);
        // 6 - 6/3 + 4 = 8
        assert_eq!(sum.push(4.0), Some(8.0));
    }

    #[test]
    fn wilder_sum_can_seed_over_fewer_bars_than_it_decays_by() {
        // The shape the directional movement family needs: seed over two
        // movements, then decay by three.
        let mut sum = WilderSum::with_seed(2, 3);
        assert_eq!(sum.push(1.0), None);
        assert_eq!(sum.push(1.0), Some(2.0));
        assert_eq!(sum.push(0.0), Some(2.0 - 2.0 / 3.0));
    }

    #[test]
    fn wilder_sum_preview_equals_the_next_push() {
        let mut sum = WilderSum::new(4);
        for x in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0] {
            let previewed = sum.preview(1.25);
            let mut forked = sum.clone();
            assert_eq!(previewed, forked.push(1.25));
            sum.push(x);
        }
    }

    #[test]
    fn every_moving_average_reproduces_its_own_indicator() {
        let series: Vec<f64> = (1..=40).map(|i| i as f64 * 1.37).collect();
        for (name, kind) in MaType::ALL {
            let mut dispatch = kind.state(5);
            let mut direct: Box<dyn FnMut(f64) -> Option<f64>> = match *kind {
                MaType::Sma => {
                    let mut inner = RollingMean::new(5);
                    Box::new(move |x| inner.push(x))
                }
                MaType::Ema => {
                    let mut inner = Ema::new(5);
                    Box::new(move |x| inner.push(x))
                }
                MaType::Wma => {
                    let mut inner = WeightedMean::new(5);
                    Box::new(move |x| inner.push(x))
                }
                MaType::Rma => {
                    let mut inner = Wilder::new(5);
                    Box::new(move |x| inner.push(x))
                }
                _ => continue,
            };
            for &value in &series {
                assert_eq!(dispatch.push(value), direct(value), "{name}");
            }
        }
    }

    #[test]
    fn moving_average_preview_equals_the_next_push() {
        let series: Vec<f64> = (1..=40).map(|i| (i as f64).sin() * 10.0 + 50.0).collect();
        for (name, kind) in MaType::ALL {
            let mut average = kind.state(4);
            for &value in &series {
                let previewed = average.preview(7.5);
                let mut forked = average.clone();
                assert_eq!(previewed, forked.push(7.5), "{name}");
                average.push(value);
            }
        }
    }

    #[test]
    fn moving_average_lookbacks_match_the_indicators() {
        assert_eq!(MaType::Sma.lookback(30), 29);
        assert_eq!(MaType::Dema.lookback(30), 58);
        assert_eq!(MaType::Tema.lookback(30), 87);
        assert_eq!(MaType::Trima.lookback(30), 29);
        assert_eq!(MaType::Kama.lookback(30), 30);
        assert_eq!(MaType::Mama.lookback(30), 32);
        assert_eq!(MaType::T3.lookback(30), 174);
        assert_eq!(MaType::Hma.lookback(30), 33);
        assert_eq!(MaType::Zlema.lookback(30), 43);
        assert_eq!(MaType::from_name("rma"), Some(MaType::Rma));
        // Every approved average is built, so an unknown name is a typo and
        // the error lists what there is.
        assert!(MaType::PENDING.is_empty());
        assert!(
            MaType::parse("ma", "ma_type", "nonsense")
                .unwrap_err()
                .to_string()
                .contains("must be one of")
        );
    }

    #[test]
    fn lagged_returns_the_value_that_many_bars_ago() {
        let mut lagged = Lagged::new(3);
        assert_eq!(drive(3, |x| lagged.push(x)), vec![None, None, None]);
        assert_eq!(lagged.push(4.0), Some(1.0));
        assert_eq!(lagged.push(5.0), Some(2.0));
    }

    #[test]
    fn lagged_earlier_matches_the_next_push() {
        let mut lagged = Lagged::new(4);
        for x in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0] {
            let seen = lagged.earlier();
            let mut forked = lagged.clone();
            assert_eq!(seen, forked.push(99.0));
            lagged.push(x);
        }
    }

    #[test]
    fn weighted_mean_weights_the_newest_bar_most() {
        let mut wma = WeightedMean::new(3);
        // (1*1 + 2*2 + 3*3) / 6 = 14 / 6
        assert_eq!(
            drive(3, |x| wma.push(x)),
            vec![None, None, Some(14.0 / 6.0)]
        );
        // (2*1 + 3*2 + 4*3) / 6 = 20 / 6
        assert_eq!(wma.push(4.0), Some(20.0 / 6.0));
    }

    #[test]
    fn weighted_mean_of_period_one_is_the_input() {
        let mut wma = WeightedMean::new(1);
        assert_eq!(
            drive(3, |x| wma.push(x)),
            vec![Some(1.0), Some(2.0), Some(3.0)]
        );
    }

    #[test]
    fn weighted_mean_preview_equals_the_next_push() {
        let mut wma = WeightedMean::new(4);
        for x in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0] {
            let previewed = wma.preview(2.25);
            let mut forked = wma.clone();
            assert_eq!(previewed, forked.push(2.25));
            wma.push(x);
        }
    }

    #[test]
    fn true_range_needs_a_previous_close() {
        let mut tr = TrueRange::new();
        assert_eq!(tr.push(11.0, 9.0, 10.0), None);
        // the gap up from 10 beats today's own 1-wide range
        assert_eq!(tr.push(13.0, 12.0, 12.5), Some(3.0));
        // today's range beats both gaps
        assert_eq!(tr.push(20.0, 10.0, 15.0), Some(10.0));
    }

    #[test]
    fn true_range_preview_equals_the_next_push() {
        let mut tr = TrueRange::new();
        tr.push(11.0, 9.0, 10.0);
        for bar in [(13.0, 12.0, 12.5), (20.0, 10.0, 15.0), (16.0, 14.0, 15.5)] {
            let previewed = tr.preview(bar.0, bar.1);
            let mut forked = tr.clone();
            assert_eq!(previewed, forked.push(bar.0, bar.1, bar.2));
            tr.push(bar.0, bar.1, bar.2);
        }
    }

    #[test]
    fn ema_of_period_one_is_the_input() {
        let mut ema = Ema::new(1);
        assert_eq!(
            drive(3, |x| ema.push(x)),
            vec![Some(1.0), Some(2.0), Some(3.0)]
        );
    }

    #[test]
    fn ema_seeds_with_the_simple_mean() {
        let mut ema = Ema::new(3);
        let got = drive(4, |x| ema.push(x));
        assert_eq!(got[2], Some(2.0));
        assert_eq!(got[3], Some(2.0 + (4.0 - 2.0) * 0.5));
    }

    #[test]
    fn ema_preview_equals_the_next_push() {
        let mut ema = Ema::new(3);
        for x in [1.0, 2.0, 3.0, 4.0, 5.0] {
            let previewed = ema.preview(7.25);
            let mut forked = ema.clone();
            assert_eq!(previewed, forked.push(7.25));
            ema.push(x);
        }
    }

    #[test]
    fn wilder_seeds_with_the_simple_mean_then_smooths() {
        let mut wilder = Wilder::new(2);
        let got = drive(3, |x| wilder.push(x));
        assert_eq!(got[1], Some(1.5));
        assert_eq!(got[2], Some(1.5 * 1.0 / 2.0 + 3.0 / 2.0));
    }

    #[test]
    fn wilder_preview_equals_the_next_push() {
        let mut wilder = Wilder::new(5);
        for x in [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0] {
            let previewed = wilder.preview(0.75);
            let mut forked = wilder.clone();
            assert_eq!(previewed, forked.push(0.75));
            wilder.push(x);
        }
    }
}

/// The moving averages an indicator can be asked to use.
///
/// `spec.yaml` names these in lowercase and the uppercase TA-Lib aliases accept
/// TA-Lib's integers for them (D12). A value whose indicator has not shipped
/// yet is rejected by name rather than quietly standing in for another average.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaType {
    Sma,
    Ema,
    Wma,
    Dema,
    Tema,
    Trima,
    Kama,
    Mama,
    T3,
    Hma,
    Zlema,
    Rma,
}

impl MaType {
    /// In `_enums.yaml` order, which is TA-Lib's integer order, so the index
    /// an indicator's parameter travels as is stable as averages are added.
    pub const ALL: &'static [(&'static str, MaType)] = &[
        ("sma", MaType::Sma),
        ("ema", MaType::Ema),
        ("wma", MaType::Wma),
        ("dema", MaType::Dema),
        ("tema", MaType::Tema),
        ("trima", MaType::Trima),
        ("kama", MaType::Kama),
        ("mama", MaType::Mama),
        ("t3", MaType::T3),
        ("hma", MaType::Hma),
        ("zlema", MaType::Zlema),
        ("rma", MaType::Rma),
    ];

    /// Values `INDICATORS.md` approves that no kernel builds yet. They are
    /// named in the error rather than treated as unknown, so a caller asking
    /// for one is told it is coming, not that it was a typo, and the enum never
    /// quietly falls back to a different average.
    pub const PENDING: &'static [&'static str] = &[];

    /// The average `name` asks for, or the error the Python layer reports.
    pub fn parse(indicator: &str, param: &str, name: &str) -> Result<Self, TlError> {
        if let Some(kind) = Self::from_name(name) {
            return Ok(kind);
        }
        let known: Vec<&str> = Self::ALL.iter().map(|(name, _)| *name).collect();
        Err(TlError::InvalidInput(if Self::PENDING.contains(&name) {
            format!(
                "{indicator}: {param}={name:?} is not implemented yet; \
                 the averages available now are {}",
                known.join(", ")
            )
        } else {
            format!(
                "{indicator}: {param} must be one of {}, got {name:?}",
                known.join(", ")
            )
        }))
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .find(|(known, _)| *known == name)
            .map(|(_, kind)| *kind)
    }

    pub fn name(self) -> &'static str {
        Self::ALL
            .iter()
            .find(|(_, kind)| *kind == self)
            .map(|(name, _)| *name)
            .unwrap_or("sma")
    }

    /// Warm-up bars this average costs at the given period.
    pub fn lookback(self, period: usize) -> usize {
        let one = period.saturating_sub(1);
        match self {
            Self::Sma | Self::Wma | Self::Trima | Self::Rma | Self::Ema => one,
            Self::Dema => 2 * one,
            Self::Tema => 3 * one,
            Self::Kama => Adaptive::lookback(period),
            // The dispatch gives MESA no period to work from: it reads one
            // from the series itself.
            Self::Mama => Mesa::LOOKBACK,
            Self::T3 => Tillson::lookback(period),
            Self::Hma => Hull::lookback(period),
            Self::Zlema => ZeroLag::lookback(period),
        }
    }

    pub fn state(self, period: usize) -> MovingAverage {
        match self {
            Self::Sma => MovingAverage::Simple(RollingMean::new(period)),
            Self::Ema => MovingAverage::Exponential(Ema::new(period)),
            Self::Wma => MovingAverage::Weighted(WeightedMean::new(period)),
            Self::Rma => MovingAverage::Smoothed(Wilder::new(period)),
            Self::Dema => MovingAverage::Double(Ema::new(period), Ema::new(period)),
            Self::Tema => {
                MovingAverage::Triple(Ema::new(period), Ema::new(period), Ema::new(period))
            }
            Self::Kama => MovingAverage::Kaufman(Adaptive::new(period)),
            // Reached through the dispatch, MESA takes the limits TA-Lib fixes
            // for it rather than ones the caller chooses, and ignores the
            // period entirely.
            Self::Mama => MovingAverage::Mesa(Box::new(Mesa::new(
                Mesa::FAST_LIMIT_DEFAULT,
                Mesa::SLOW_LIMIT_DEFAULT,
            ))),
            // Reached through the dispatch, T3 takes the `v_factor` TA-Lib
            // fixes for it rather than one the caller chooses.
            Self::T3 => MovingAverage::Tillson(Tillson::new(period, Tillson::V_FACTOR_DEFAULT)),
            Self::Hma => MovingAverage::Hull(Hull::new(period)),
            Self::Zlema => MovingAverage::ZeroLag(ZeroLag::new(period)),
            Self::Trima => {
                // Averaging twice over halves of the window is what puts the
                // triangular weighting in; an even period gives the extra bar
                // to the second stage, which is where TA-Lib puts it.
                let half = period / 2;
                let (first, second) = if period % 2 == 1 {
                    (half + 1, half + 1)
                } else {
                    (half, half + 1)
                };
                MovingAverage::Triangular(RollingMean::new(first), RollingMean::new(second))
            }
        }
    }
}

/// One moving average, chosen at construction and then driven like any other
/// step function.
#[derive(Clone, Debug)]
pub enum MovingAverage {
    Simple(RollingMean),
    Exponential(Ema),
    Weighted(WeightedMean),
    Smoothed(Wilder),
    Double(Ema, Ema),
    Triple(Ema, Ema, Ema),
    Triangular(RollingMean, RollingMean),
    Kaufman(Adaptive),
    // MESA carries the whole Hilbert transform, which is far larger than the
    // other averages; boxing it keeps the enum the size of the rest.
    Mesa(Box<Mesa>),
    Tillson(Tillson),
    Hull(Hull),
    ZeroLag(ZeroLag),
}

impl MovingAverage {
    pub fn push(&mut self, value: f64) -> Option<f64> {
        match self {
            Self::Simple(inner) => inner.push(value),
            Self::Exponential(inner) => inner.push(value),
            Self::Weighted(inner) => inner.push(value),
            Self::Smoothed(inner) => inner.push(value),
            Self::Double(first, second) => {
                let one = first.push(value)?;
                second.push(one).map(|two| 2.0 * one - two)
            }
            Self::Triple(first, second, third) => {
                let one = first.push(value)?;
                let two = second.push(one)?;
                third.push(two).map(|three| 3.0 * one - 3.0 * two + three)
            }
            Self::Triangular(first, second) => {
                let one = first.push(value)?;
                second.push(one)
            }
            Self::Kaufman(inner) => inner.push(value),
            // Only the MAMA line is a moving average; FAMA is a second
            // reading the dispatch does not expose.
            Self::Mesa(inner) => inner.push(value).map(|(mama, _)| mama),
            Self::Tillson(inner) => inner.push(value),
            Self::Hull(inner) => inner.push(value),
            Self::ZeroLag(inner) => inner.push(value),
        }
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        match self {
            Self::Simple(inner) => inner.preview(value),
            Self::Exponential(inner) => inner.preview(value),
            Self::Weighted(inner) => inner.preview(value),
            Self::Smoothed(inner) => inner.preview(value),
            Self::Double(first, second) => {
                let one = first.preview(value)?;
                second.preview(one).map(|two| 2.0 * one - two)
            }
            Self::Triple(first, second, third) => {
                let one = first.preview(value)?;
                let two = second.preview(one)?;
                third
                    .preview(two)
                    .map(|three| 3.0 * one - 3.0 * two + three)
            }
            Self::Triangular(first, second) => {
                let one = first.preview(value)?;
                second.preview(one)
            }
            Self::Kaufman(inner) => inner.preview(value),
            Self::Mesa(inner) => inner.preview(value).map(|(mama, _)| mama),
            Self::Tillson(inner) => inner.preview(value),
            Self::Hull(inner) => inner.preview(value),
            Self::ZeroLag(inner) => inner.preview(value),
        }
    }
}
