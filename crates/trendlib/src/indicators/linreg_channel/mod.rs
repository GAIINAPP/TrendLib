use crate::core::chart::{at_least, whole};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::{Fit, LinearRegression, RollingWindow};

pub const NAME: &str = "linreg_channel";

pub const PERIOD_DEFAULT: usize = 100;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100_000;
pub const DEVIATION_DEFAULT: f64 = 2.0;
pub const DEVIATION_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub period: usize,
    pub deviation: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            period: PERIOD_DEFAULT,
            deviation: DEVIATION_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    line: LinearRegression,
    window: RollingWindow,
    deviation: f64,
}

impl State {
    fn bands(&self, fit: Fit, values: impl Iterator<Item = f64>) -> [f64; 3] {
        let mut squares = 0.0;
        for (offset, value) in values.enumerate() {
            let residual = value - (fit.intercept + fit.slope * offset as f64);
            squares += residual * residual;
        }
        let degrees = (self.window.period() - 1) as f64;
        let spread = (squares / degrees).sqrt() * self.deviation;
        let middle = fit.at_last_bar();
        [middle + spread, middle, middle - spread]
    }
}

impl Step<1, 3> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 3]> {
        self.window.push(bar[0]);
        let fit = self.line.push(bar[0])?;
        Some(self.bands(fit, self.window.values()))
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 3]> {
        let fit = self.line.preview(bar[0])?;
        Some(self.bands(fit, self.window.preview_values(bar[0])))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LinregChannel;

pub type LinregChannelStream = BarStream<LinregChannel, 1, 3>;

impl Kernel<1, 3> for LinregChannel {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 3] = [
        "linreg_channel_upper",
        "linreg_channel_middle",
        "linreg_channel_lower",
    ];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(NAME, "period", params.period, PERIOD_MIN, PERIOD_MAX)?;
        at_least(NAME, "deviation", params.deviation, DEVIATION_MIN)?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.period - 1
    }

    fn state(params: &Params) -> State {
        State {
            line: LinearRegression::new(params.period),
            window: RollingWindow::new(params.period),
            deviation: params.deviation,
        }
    }
}
