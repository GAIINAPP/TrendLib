use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "percentrank";

pub const PERIOD_DEFAULT: usize = 100;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    /// The `period` bars before the current one: it is read before the bar
    /// is added, so the bar is ranked against its history rather than against
    /// a window it is itself part of. That is what makes a flat stretch read
    /// 0 rather than 100.
    history: RollingWindow,
}

/// Strictly below, so a value equal to one in the window does not count. That
/// is TA-Lib's comparison, and it is what makes a flat series read 0.
fn rank(history: &RollingWindow, value: f64) -> f64 {
    let below = history.values().filter(|held| *held < value).count();
    100.0 * below as f64 / history.period() as f64
}

impl Step<1, 1> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 1]> {
        let out = self
            .history
            .is_full()
            .then(|| [rank(&self.history, bar[0])]);
        self.history.push(bar[0]);
        out
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 1]> {
        self.history
            .is_full()
            .then(|| [rank(&self.history, bar[0])])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Percentrank;

pub type PercentrankStream = BarStream<Percentrank, 1, 1>;

impl Kernel<1, 1> for Percentrank {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 1] = ["percentrank"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(PERIOD_MIN..=PERIOD_MAX).contains(&params.period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "period",
                params.period,
                PERIOD_MIN,
                PERIOD_MAX,
            ));
        }
        Ok(())
    }

    // The bar is ranked against the `period` bars before it, so the first
    // ranked bar is one further in than a window that included it.
    fn lookback(params: &Params) -> usize {
        params.period
    }

    fn state(params: &Params) -> State {
        State {
            history: RollingWindow::new(params.period),
        }
    }
}
