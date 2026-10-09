use crate::core::candles::{CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_breakaway";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 14;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// A gap away from a long bar, three bars drifting further, then one closing back into the gap
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let fourth = pattern.back(1);
    let third = pattern.back(2);
    let second = pattern.back(3);
    let first = pattern.back(4);
    let ran_down = first.is_black()
        && second.body_gaps_below(first)
        && third.high < second.high
        && third.low < second.low
        && fourth.high < third.high
        && fourth.low < third.low
        && bar.close > second.open
        && bar.close < first.close;
    let ran_up = first.is_white()
        && second.body_gaps_above(first)
        && third.high > second.high
        && third.low > second.low
        && fourth.high > third.high
        && fourth.low > third.low
        && bar.close < second.open
        && bar.close > first.close;
    if first.body() > pattern.average(CandleSetting::BODY_LONG, 4)
        && first.is_white() == second.is_white()
        && second.is_white() == fourth.is_white()
        && fourth.is_white() != bar.is_white()
        && (ran_down || ran_up)
    {
        bar.colour() * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CdlBreakaway;

pub type CdlBreakawayStream = BarStream<CdlBreakaway, 4, 1>;

impl Kernel<4, 1> for CdlBreakaway {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_breakaway"];

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
