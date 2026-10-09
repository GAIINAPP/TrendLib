use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Ema;

pub const NAME: &str = "macdfix";

pub const SIGNAL_PERIOD_DEFAULT: usize = 9;
pub const SIGNAL_PERIOD_MIN: usize = 1;
pub const SIGNAL_PERIOD_MAX: usize = 100_000;

pub const FAST_PERIOD: usize = 12;
pub const SLOW_PERIOD: usize = 26;

/// TA-Lib writes the two smoothing constants as literals rather than deriving
/// them from the periods, and `2 / (26 + 1)` is 0.074074 while `2 / (12 + 1)`
/// is 0.153846. The fixed MACD is therefore not `macd(12, 26)`; `doc.md` says
/// so and a test pins the difference.
pub const SLOW_SMOOTHING: f64 = 0.075;
pub const FAST_SMOOTHING: f64 = 0.15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub signal_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            signal_period: SIGNAL_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    fast: Ema,
    slow: Ema,
    signal: Ema,
    seen: usize,
}

impl Step<1, 3> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 3]> {
        self.seen += 1;
        // Both averages must be offered the bar before either `?` can bail, or
        // the one that is not skipping would miss the bars the other skips.
        let fast = if self.seen > SLOW_PERIOD - FAST_PERIOD {
            self.fast.push(bar[0])
        } else {
            None
        };
        let slow = self.slow.push(bar[0]);
        let line = fast? - slow?;
        let signal = self.signal.push(line)?;
        Some([line, signal, line - signal])
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 3]> {
        let fast = (self.seen + 1 > SLOW_PERIOD - FAST_PERIOD)
            .then(|| self.fast.preview(bar[0]))
            .flatten()?;
        let slow = self.slow.preview(bar[0])?;
        let line = fast - slow;
        let signal = self.signal.preview(line)?;
        Some([line, signal, line - signal])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Macdfix;

pub type MacdfixStream = BarStream<Macdfix, 1, 3>;

impl Kernel<1, 3> for Macdfix {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 3] = ["macdfix_macd", "macdfix_signal", "macdfix_hist"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(SIGNAL_PERIOD_MIN..=SIGNAL_PERIOD_MAX).contains(&params.signal_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "signal_period",
                params.signal_period,
                SIGNAL_PERIOD_MIN,
                SIGNAL_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        SLOW_PERIOD - 1 + params.signal_period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            // The shorter average starts late, by the difference in periods,
            // so both reach their first value on the same bar.
            fast: Ema::with_smoothing(FAST_PERIOD, FAST_SMOOTHING),
            slow: Ema::with_smoothing(SLOW_PERIOD, SLOW_SMOOTHING),
            signal: Ema::new(params.signal_period),
            seen: 0,
        }
    }
}
