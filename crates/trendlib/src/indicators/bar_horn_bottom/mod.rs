use crate::core::bars::{BarHistory, BarState};
use crate::core::chart::EPS;
use crate::core::chart::at_least;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_horn_bottom";

pub const TOL_DEFAULT: f64 = 0.02;
pub const TOL_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub tol: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self { tol: TOL_DEFAULT }
    }
}

pub type State = BarState<Params>;

/// Two dips two bars apart at about the same price, with a
/// higher bar between them.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let (bar, middle, outer) = (history.back(0), history.back(1), history.back(2));
    if (bar.low - outer.low).abs() / (bar.low.abs().max(outer.low.abs()) + EPS) < params.tol
        && middle.low > bar.low.max(outer.low) * 1.01
        && bar.close > middle.close
    {
        100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BarHornBottom;

pub type BarHornBottomStream = BarStream<BarHornBottom, 4, 1>;

impl Kernel<4, 1> for BarHornBottom {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_horn_bottom"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        at_least(NAME, "tol", params.tol, TOL_MIN)?;
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        2
    }

    fn state(params: &Params) -> State {
        BarState::new(2, 2, *params, rule)
    }
}
