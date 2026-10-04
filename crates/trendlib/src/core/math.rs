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
    use super::{Ema, RollingMean, TrueRange, WeightedMean, Wilder};

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
