use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::RollingWindow;

pub const NAME: &str = "cmf";

pub const PERIOD_DEFAULT: usize = 20;
pub const PERIOD_MIN: usize = 2;
pub const PERIOD_MAX: usize = 100000;

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
    flows: RollingWindow,
    volumes: RollingWindow,
}

/// Where the close sat in its own bar, from -1 at the low to +1 at the high,
/// times the volume. A bar with no range says nothing about where buyers won,
/// so it contributes nothing rather than dividing by zero.
fn flow(bar: [f64; 4]) -> f64 {
    let (high, low, close, volume) = (bar[0], bar[1], bar[2], bar[3]);
    let span = high - low;
    if span == 0.0 {
        0.0
    } else {
        ((close - low) - (high - close)) / span * volume
    }
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let full = self.flows.push(flow(bar));
        self.volumes.push(bar[3]);
        full.then(|| [self.flows.values().sum::<f64>() / self.volumes.values().sum::<f64>()])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        self.flows.preview_is_full().then(|| {
            [self.flows.preview_values(flow(bar)).sum::<f64>()
                / self.volumes.preview_values(bar[3]).sum::<f64>()]
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cmf;

pub type CmfStream = BarStream<Cmf, 4, 1>;

impl Kernel<4, 1> for Cmf {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["high", "low", "close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["cmf"];

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
            flows: RollingWindow::new(params.period),
            volumes: RollingWindow::new(params.period),
        }
    }
}
