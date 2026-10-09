use crate::core::bars::{BarHistory, BarState};
use crate::core::chart::EPS;
use crate::core::chart::at_least;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_horn_top";

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

/// Two spikes two bars apart at about the same price, with a
/// lower bar between them.
fn rule(history: &BarHistory, params: &Params) -> f64 {
    let (bar, middle, outer) = (history.back(0), history.back(1), history.back(2));
    if (bar.high - outer.high).abs() / (bar.high.max(outer.high) + EPS) < params.tol
        && middle.high < bar.high.min(outer.high) * 0.99
        && bar.close < middle.close
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BarHornTop;

pub type BarHornTopStream = BarStream<BarHornTop, 4, 1>;

impl Kernel<4, 1> for BarHornTop {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_horn_top"];

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
