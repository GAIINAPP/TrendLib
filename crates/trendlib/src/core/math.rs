//! Shared kernels.
//!
//! Each kernel is one step function used by both the batch loop and the stream,
//! so `docs/CONVENTIONS.md` § 1 (identical arithmetic in both paths) holds by
//! construction rather than by review. `push` commits a bar; `preview` returns
//! what the next `push` would return and touches nothing.

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
/// The weighted sum is advanced rather than rebuilt: adding `period * newest`
/// and then subtracting the plain window sum shifts every older weight down by
/// one, which is the identity `W(t+1) = period * x(t+1) + W(t) - S(t)`. Without
/// it a batch run would be O(bars * period) and could not meet the speed
/// requirement in `SPEC.md` § 6.
#[derive(Clone, Debug)]
pub struct WeightedMean {
    window: Box<[f64]>,
    period: f64,
    divider: f64,
    head: usize,
    seen: usize,
    weighted: f64,
    total: f64,
}

impl WeightedMean {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "period must be at least 1");
        let period_f64 = period as f64;
        Self {
            window: vec![0.0; period].into_boxed_slice(),
            period: period_f64,
            divider: period_f64 * (period_f64 + 1.0) / 2.0,
            head: 0,
            seen: 0,
            weighted: 0.0,
            total: 0.0,
        }
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        self.total += value;
        self.seen += 1;
        self.window[self.head] = value;
        self.head = (self.head + 1) % self.window.len();
        if self.seen < self.window.len() {
            self.weighted += value * self.seen as f64;
            return None;
        }
        self.weighted += value * self.period;
        let mean = self.weighted / self.divider;
        self.weighted -= self.total;
        self.total -= self.window[self.head];
        Some(mean)
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        if self.seen + 1 < self.window.len() {
            return None;
        }
        Some((self.weighted + value * self.period) / self.divider)
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

    fn range(high: f64, low: f64, previous_close: f64) -> f64 {
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
            head: 0,
            seen: 0,
            best: 0,
            want_max,
        }
    }

    fn beats(&self, candidate: f64, incumbent: f64) -> bool {
        if self.want_max {
            candidate > incumbent
        } else {
            candidate < incumbent
        }
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
            match best {
                None => best = Some(slot),
                Some(current) if self.beats(self.window[slot], self.window[current]) => {
                    best = Some(slot)
                }
                _ => {}
            }
        }
        best
    }

    pub fn push(&mut self, value: f64) -> Option<f64> {
        let period = self.window.len();
        let slot = self.head;
        let dropped_record = self.seen >= period && slot == self.best;

        self.window[slot] = value;
        self.head = (self.head + 1) % period;
        self.seen += 1;

        if dropped_record {
            self.best = self.scan(period, None).expect("a full window has a best");
        } else if self.seen == 1 || self.beats(value, self.window[self.best]) {
            self.best = slot;
        }

        (self.seen >= period).then(|| self.window[self.best])
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        let period = self.window.len();
        if self.seen + 1 < period {
            return None;
        }
        let incumbent = if self.seen < period {
            // The window fills exactly on this bar, so nothing drops out.
            self.scan(self.seen, None)
        } else if self.head == self.best {
            self.scan(period, Some(self.head))
        } else {
            Some(self.best)
        };
        let Some(incumbent) = incumbent.map(|slot| self.window[slot]) else {
            return Some(value);
        };
        Some(if self.beats(value, incumbent) {
            value
        } else {
            incumbent
        })
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

#[cfg(test)]
mod tests {
    use super::{
        Ema, Lagged, RollingExtreme, RollingMean, RollingWindow, TrueRange, WeightedMean, Wilder,
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
