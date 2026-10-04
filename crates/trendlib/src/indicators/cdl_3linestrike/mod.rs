use crate::core::candles::{Candle, CandleSetting, PatternState};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "cdl_3linestrike";

/// Taken from the oracle, which reserves the longest average any of the
/// settings below uses plus the bars the pattern reads behind the last one.
pub const LOOKBACK: usize = 8;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = PatternState;

/// Three bars running one way, then one that opens past the last and closes past the first
fn detect(pattern: &PatternState) -> f64 {
    let bar = pattern.back(0);
    let third = pattern.back(1);
    let second = pattern.back(2);
    let first = pattern.back(3);
    let opens_within = |open: f64, candle: Candle, back: usize| {
        let near = pattern.average(CandleSetting::NEAR, back);
        open >= candle.body_bottom() - near && open <= candle.body_top() + near
    };
    let struck_up = third.is_white()
        && third.close > second.close
        && second.close > first.close
        && bar.open > third.close
        && bar.close < first.open;
    let struck_down = third.is_black()
        && third.close < second.close
        && second.close < first.close
        && bar.open < third.close
        && bar.close > first.open;
    if first.is_white() == second.is_white()
        && second.is_white() == third.is_white()
        && bar.is_white() != third.is_white()
        && opens_within(second.open, first, 3)
        && opens_within(third.open, second, 2)
        && (struck_up || struck_down)
    {
        third.colour() * 100.0
    } else {
        0.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Cdl3linestrike;

pub type Cdl3linestrikeStream = BarStream<Cdl3linestrike, 4, 1>;

impl Kernel<4, 1> for Cdl3linestrike {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["cdl_3linestrike"];

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
