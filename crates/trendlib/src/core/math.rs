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
        self.prev = ((value - self.prev) * self.k) + self.prev;
        self.seen += 1;
        Some(self.prev)
    }

    pub fn preview(&self, value: f64) -> Option<f64> {
        match (self.seen + 1).cmp(&self.period) {
            std::cmp::Ordering::Less => None,
            std::cmp::Ordering::Equal => Some((self.seed_total + value) / self.period_f64),
            std::cmp::Ordering::Greater => Some(((value - self.prev) * self.k) + self.prev),
        }
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
    use super::{Ema, RollingMean, Wilder};

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
