use crate::core::bars::{BarHistory, BarState, combine};
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel};

pub const NAME: &str = "bar_fakey";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

pub type State = BarState<Params>;

/// An inside bar, then a false break of it: the next bar trades through one
/// side of the inside bar and closes back across that side, still within the
/// mother bar. It is read against the failed break, so a false break down is
/// +100 and a false break up is -100. A bar that breaks both sides says
/// neither, and a close beyond the mother bar is a real breakout, not a fakey.
fn rule(history: &BarHistory, _params: &Params) -> f64 {
    let (bar, inner, mother) = (history.back(0), history.back(1), history.back(2));
    let inside = inner.high < mother.high && inner.low > mother.low;
    let failed_down = bar.low < inner.low && bar.close > inner.low && bar.close < mother.high;
    let failed_up = bar.high > inner.high && bar.close < inner.high && bar.close > mother.low;
    combine(inside && failed_down, inside && failed_up)
}

#[derive(Clone, Copy, Debug)]
pub struct BarFakey;

pub type BarFakeyStream = BarStream<BarFakey, 4, 1>;

impl Kernel<4, 1> for BarFakey {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["open", "high", "low", "close"];
    const OUTPUTS: [&'static str; 1] = ["bar_fakey"];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        2
    }

    fn state(params: &Params) -> State {
        BarState::new(2, 2, *params, rule)
    }
}

#[cfg(test)]
mod tests {
    use super::{BarFakey, Params};
    use crate::core::kernel::Kernel;

    /// Mother bar 100-110, inside bar 103-107, then `last` as (open, high, low, close).
    fn fakey(last: (f64, f64, f64, f64)) -> f64 {
        let open = [105.0, 105.0, last.0];
        let high = [110.0, 107.0, last.1];
        let low = [100.0, 103.0, last.2];
        let close = [106.0, 104.0, last.3];
        let [out] = BarFakey::batch([&open, &high, &low, &close], &Params).unwrap();
        out[2]
    }

    #[test]
    fn a_false_break_down_is_bullish() {
        assert_eq!(fakey((104.0, 105.0, 101.5, 104.5)), 100.0);
    }

    #[test]
    fn a_false_break_up_is_bearish() {
        assert_eq!(fakey((106.0, 108.5, 105.0, 105.5)), -100.0);
    }

    #[test]
    fn a_real_breakout_is_not_a_fakey() {
        assert_eq!(fakey((107.0, 112.0, 106.0, 111.0)), 0.0);
        assert_eq!(fakey((103.0, 104.0, 98.0, 99.0)), 0.0);
    }

    #[test]
    fn a_break_that_has_not_failed_yet_is_not_a_fakey() {
        assert_eq!(fakey((104.0, 104.5, 101.0, 102.0)), 0.0);
    }

    #[test]
    fn breaking_both_sides_says_neither() {
        assert_eq!(fakey((105.0, 108.0, 102.0, 105.0)), 0.0);
    }

    #[test]
    fn without_an_inside_bar_there_is_no_fakey() {
        let open = [105.0, 105.0, 104.0];
        let high = [110.0, 111.0, 105.0];
        let low = [100.0, 103.0, 101.5];
        let close = [106.0, 104.0, 104.5];
        let [out] = BarFakey::batch([&open, &high, &low, &close], &Params).unwrap();
        assert_eq!(out[2], 0.0);
    }
}
