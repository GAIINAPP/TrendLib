use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_gapsidesidewhite";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 7;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Two white bars of the same size opening at the same price, both on the far side of a gap
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let middle = pattern.back(1);
    let first = pattern.back(2);
    let gapped_up = middle.body_gaps_above(first) && bar.body_gaps_above(first);
    let gapped_down = middle.body_gaps_below(first) && bar.body_gaps_below(first);
    let near = pattern.average(CandleSetting::NEAR, 1);
    let equal = pattern.average(CandleSetting::EQUAL, 1);
    if (gapped_up || gapped_down)
        && middle.is_white()
        && bar.is_white()
        && bar.body() >= middle.body() - near
        && bar.body() <= middle.body() + near
        && bar.open >= middle.open - equal
        && bar.open <= middle.open + equal
    {
        if gapped_up { 100.0 } else { -100.0 }
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlGapsidesidewhite;

pub type CdlGapsidesidewhiteStream = BarStream<CdlGapsidesidewhite, 4, 1>;

impl Kernel<4, 1> for CdlGapsidesidewhite {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_gapsidesidewhite"];

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
