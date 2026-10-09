use crate::core::candles::PatternState;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_xsidegap3methods";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Two bars of one colour with a gap between them, then a third that closes the gap
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let prior = pattern.back(1);
    let first = pattern.back(2);
    let ran_on = (first.is_white() && prior.is_white() && prior.body_gaps_above(first))
        || (first.is_black() && prior.is_black() && prior.body_gaps_below(first));
    if ran_on
        && bar.is_white() != prior.is_white()
        && bar.open < prior.body_top()
        && bar.open > prior.body_bottom()
        && bar.close < first.body_top()
        && bar.close > first.body_bottom()
    {
        prior.colour() * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlXsidegap3methods;

pub type CdlXsidegap3methodsStream = BarStream<CdlXsidegap3methods, 4, 1>;

impl Kernel<4, 1> for CdlXsidegap3methods {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_xsidegap3methods"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        LOOKBACK
    }

    fn state(params: &Params) -> State {
        let _ = params;
        PatternState::new(LOOKBACK, 0.0, detect)
    }
}
