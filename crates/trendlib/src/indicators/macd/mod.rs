use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Ema;

pub const NAME: &str = "macd";

pub const FAST_PERIOD_DEFAULT: usize = 12;
pub const SLOW_PERIOD_DEFAULT: usize = 26;
pub const SIGNAL_PERIOD_DEFAULT: usize = 9;
pub const PERIOD_MIN: usize = 2;
pub const SIGNAL_PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            fast_period: FAST_PERIOD_DEFAULT,
            slow_period: SLOW_PERIOD_DEFAULT,
            signal_period: SIGNAL_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    fast: Ema,
    slow: Ema,
    signal: Ema,
    /// Bars each average ignores before it starts gathering its seed.
    fast_skip: usize,
    slow_skip: usize,
    seen: usize,
}

impl Step<1, 3> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 3]> {
        self.seen += 1;
        // Both averages must be offered the bar before either `?` can bail,
        // or the one that is not skipping would miss the bars the other skips.
        let fast = if self.seen > self.fast_skip {
            self.fast.push(bar[0])
        } else {
            None
        };
        let slow = if self.seen > self.slow_skip {
            self.slow.push(bar[0])
        } else {
            None
        };
        let line = fast? - slow?;
        let signal = self.signal.push(line)?;
        Some([line, signal, line - signal])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 3]> {
        let next = self.seen + 1;
        let fast = (next > self.fast_skip)
            .then(|| self.fast.preview(bar[0]))
            .flatten()?;
        let slow = (next > self.slow_skip)
            .then(|| self.slow.preview(bar[0]))
            .flatten()?;
        let line = fast - slow;
        let signal = self.signal.preview(line)?;
        Some([line, signal, line - signal])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Macd;

pub type MacdStream = BarStream<Macd, 1, 3>;

impl Kernel<1, 3> for Macd {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 3] = ["macd", "macd_signal", "macd_hist"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        for (name, value, min) in [
            ("fast_period", params.fast_period, PERIOD_MIN),
            ("slow_period", params.slow_period, PERIOD_MIN),
            ("signal_period", params.signal_period, SIGNAL_PERIOD_MIN),
        ] {
            if !(min..=PERIOD_MAX).contains(&value) {
                return Err(TlError::param_out_of_range(
                    NAME, name, value, min, PERIOD_MAX,
                ));
            }
        }
        Ok(())
    }

    // The longer average decides when the difference starts, and the signal
    // average then needs that many differences.
    fn lookback(params: &Params) -> usize {
        let longest = params.fast_period.max(params.slow_period);
        longest.saturating_sub(1) + params.signal_period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        // The shorter average is started late, by exactly the difference in
        // periods, so both reach their first value on the same bar. Running
        // both from bar one instead would give the faster average a longer
        // history at every point and shift the whole line.
        let longest = params.fast_period.max(params.slow_period);
        State {
            fast: Ema::new(params.fast_period),
            slow: Ema::new(params.slow_period),
            signal: Ema::new(params.signal_period),
            fast_skip: longest - params.fast_period,
            slow_skip: longest - params.slow_period,
            seen: 0,
        }
    }
}
