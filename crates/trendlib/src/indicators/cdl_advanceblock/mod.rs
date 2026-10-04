use crate::core::candles::{Candle, CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_advanceblock";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 12;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Three white bars closing higher, with the advance visibly slowing
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let second = pattern.back(1);
    let first = pattern.back(2);
    let opens_within = |open: f64, previous: Candle, back: usize| {
        open > previous.open && open <= previous.close + pattern.average(CandleSetting::NEAR, back)
    };
    let losing_steam = (second.body() < first.body() - pattern.average(CandleSetting::FAR, 2)
        && bar.body() < second.body() + pattern.average(CandleSetting::NEAR, 1))
        || bar.body() < second.body() - pattern.average(CandleSetting::FAR, 1)
        || (bar.body() < second.body()
            && second.body() < first.body()
            && (bar.upper_shadow() > pattern.average(CandleSetting::SHADOW_SHORT, 0)
                || second.upper_shadow() > pattern.average(CandleSetting::SHADOW_SHORT, 1)))
        || (bar.body() < second.body()
            && bar.upper_shadow() > pattern.average(CandleSetting::SHADOW_LONG, 0));
    if first.is_white()
        && second.is_white()
        && bar.is_white()
        && bar.close > second.close
        && second.close > first.close
        && opens_within(second.open, first, 2)
        && opens_within(bar.open, second, 1)
        && first.body() > pattern.average(CandleSetting::BODY_LONG, 2)
        && first.upper_shadow() < pattern.average(CandleSetting::SHADOW_SHORT, 2)
        && losing_steam
    {
        -100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlAdvanceblock;

pub type CdlAdvanceblockStream = BarStream<CdlAdvanceblock, 4, 1>;

impl Kernel<4, 1> for CdlAdvanceblock {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_advanceblock"];

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
