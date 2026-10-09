use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{MaType, MovingAverage};

pub const NAME: &str = "macdext";

pub const FAST_PERIOD_DEFAULT: usize = 12;
pub const SLOW_PERIOD_DEFAULT: usize = 26;
pub const SIGNAL_PERIOD_DEFAULT: usize = 9;
pub const PERIOD_MIN: usize = 2;
pub const SIGNAL_PERIOD_MIN: usize = 1;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub fast_period: usize,
    pub fast_ma_type: MaType,
    pub slow_period: usize,
    pub slow_ma_type: MaType,
    pub signal_period: usize,
    pub signal_ma_type: MaType,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            fast_period: FAST_PERIOD_DEFAULT,
            fast_ma_type: MaType::Sma,
            slow_period: SLOW_PERIOD_DEFAULT,
            slow_ma_type: MaType::Sma,
            signal_period: SIGNAL_PERIOD_DEFAULT,
            signal_ma_type: MaType::Sma,
        }
    }
}

/// When each average starts, so that both reach their first value on the same
/// bar. The two may be different kinds, so it is their warm-ups that are
/// lined up, not their periods.
fn skips(params: &Params) -> (usize, usize, usize) {
    let fast = params.fast_ma_type.lookback(params.fast_period);
    let slow = params.slow_ma_type.lookback(params.slow_period);
    let longest = fast.max(slow);
    (longest - fast, longest - slow, longest)
}

#[derive(Clone, Debug)]
pub struct State {
    fast: MovingAverage,
    slow: MovingAverage,
    signal: MovingAverage,
    fast_skip: usize,
    slow_skip: usize,
    seen: usize,
}

impl Step<1, 3> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 3]> {
        self.seen += 1;
        // Both averages must be offered the bar before either `?` can bail, or
        // the one that is not skipping would miss the bars the other skips.
        let fast = (self.seen > self.fast_skip)
            .then(|| self.fast.push(bar[0]))
            .flatten();
        let slow = (self.seen > self.slow_skip)
            .then(|| self.slow.push(bar[0]))
            .flatten();
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
pub struct Macdext;

pub type MacdextStream = BarStream<Macdext, 1, 3>;

impl Kernel<1, 3> for Macdext {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 3] = ["macdext_macd", "macdext_signal", "macdext_hist"];

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

    fn lookback(params: &Params) -> usize {
        skips(params).2 + params.signal_ma_type.lookback(params.signal_period)
    }

    fn state(params: &Params) -> State {
        let (fast_skip, slow_skip, _) = skips(params);
        State {
            fast: params.fast_ma_type.state(params.fast_period),
            slow: params.slow_ma_type.state(params.slow_period),
            signal: params.signal_ma_type.state(params.signal_period),
            fast_skip,
            slow_skip,
            seen: 0,
        }
    }
}
