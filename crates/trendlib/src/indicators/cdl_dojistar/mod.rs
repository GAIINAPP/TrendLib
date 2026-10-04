use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_dojistar";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 11;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A doji whose body gaps clear of the long body before it
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let prior = pattern.back(1);
    let jumped = (prior.is_white() && bar.body_gaps_above(prior))
        || (prior.is_black() && bar.body_gaps_below(prior));
    if prior.body() > pattern.average(CandleSetting::BODY_LONG, 1)
        && bar.body() <= pattern.average(CandleSetting::BODY_DOJI, 0)
        && jumped
    {
        -prior.colour() * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlDojistar;

pub type CdlDojistarStream = BarStream<CdlDojistar, 4, 1>;

impl Kernel<4, 1> for CdlDojistar {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_dojistar"];

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
