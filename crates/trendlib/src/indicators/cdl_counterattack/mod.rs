use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_counterattack";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 11;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Two long bars of opposite colours that closed at the same price
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let prior = pattern.back(1);
    let equal = pattern.average(CandleSetting::EQUAL, 1);
    if bar.is_white() != prior.is_white()
        && bar.body() > pattern.average(CandleSetting::BODY_LONG, 0)
        && prior.body() > pattern.average(CandleSetting::BODY_LONG, 1)
        && bar.close <= prior.close + equal
        && bar.close >= prior.close - equal
    {
        bar.colour() * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlCounterattack;

pub type CdlCounterattackStream = BarStream<CdlCounterattack, 4, 1>;

impl Kernel<4, 1> for CdlCounterattack {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_counterattack"];

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
