use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "vwma";

pub const PERIOD_DEFAULT: usize = 30;
pub const PERIOD_MIN: usize = 1;
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
    traded: RollingWindow,
    volume: RollingWindow,
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let full = self.traded.push(bar[0] * bar[1]);
        self.volume.push(bar[1]);
        if !full {
            return None;
        }
        Some([weighted(self.traded.values(), self.volume.values())])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        if !self.traded.preview_is_full() {
            return None;
        }
        Some([weighted(
            self.traded.preview_values(bar[0] * bar[1]),
            self.volume.preview_values(bar[1]),
        )])
    }
}

/// Both sums are re-added from the window rather than carried forward: the
/// quotient of two running totals drifts where a long window holds volumes of
/// very different sizes.
fn weighted(traded: impl Iterator<Item = f64>, volume: impl Iterator<Item = f64>) -> f64 {
    traded.sum::<f64>() / volume.sum::<f64>()
}

#[derive(Clone, Copy, Debug)]
pub struct Vwma;

pub type VwmaStream = BarStream<Vwma, 2, 1>;

impl Kernel<2, 1> for Vwma {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["vwma"];

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

    fn lookback(params: &Params) -> usize {
        params.period.saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            traded: RollingWindow::new(params.period),
            volume: RollingWindow::new(params.period),
        }
    }
}
