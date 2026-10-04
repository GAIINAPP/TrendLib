use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_separatinglines";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 11;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A long bar that opened where the opposite-coloured bar before it opened, and ran the other way
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let prior = pattern.back(1);
    let equal = pattern.average(CandleSetting::EQUAL, 1);
    let short_shadow = pattern.average(CandleSetting::SHADOW_VERY_SHORT, 0);
    let opened_flat = bar.open <= prior.open + equal && bar.open >= prior.open - equal;
    let holds_its_end = (bar.is_white() && bar.lower_shadow() < short_shadow)
        || (bar.is_black() && bar.upper_shadow() < short_shadow);
    if bar.is_white() != prior.is_white()
        && opened_flat
        && bar.body() > pattern.average(CandleSetting::BODY_LONG, 0)
        && holds_its_end
    {
        bar.colour() * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlSeparatinglines;

pub type CdlSeparatinglinesStream = BarStream<CdlSeparatinglines, 4, 1>;

impl Kernel<4, 1> for CdlSeparatinglines {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_separatinglines"];

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
