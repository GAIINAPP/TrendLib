use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::EmaFromFirst;
use crate::indicators::ad;

pub const NAME: &str = "adosc";

pub const FAST_PERIOD_DEFAULT: usize = 3;
pub const FAST_PERIOD_MIN: usize = 2;
pub const FAST_PERIOD_MAX: usize = 100000;
pub const SLOW_PERIOD_DEFAULT: usize = 10;
pub const SLOW_PERIOD_MIN: usize = 2;
pub const SLOW_PERIOD_MAX: usize = 100000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub fast_period: usize,
    pub slow_period: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            fast_period: FAST_PERIOD_DEFAULT,
            slow_period: SLOW_PERIOD_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    line: ad::State,
    fast: EmaFromFirst,
    slow: EmaFromFirst,
    /// Neither average has a warm-up of its own, so the rows before the longer
    /// period has passed are held back by counting them.
    seen: usize,
    warm_up: usize,
}

impl Step<4, 1> for State {
    fn push(&mut self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let [line] = self.line.push(bar)?;
        let value = self.fast.push(line) - self.slow.push(line);
        self.seen += 1;
        (self.seen > self.warm_up).then_some([value])
    }

    fn preview(&self, bar: [f64; 4]) -> Option<[f64; 1]> {
        let [line] = self.line.preview(bar)?;
        let value = self.fast.preview(line) - self.slow.preview(line);
        (self.seen + 1 > self.warm_up).then_some([value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Adosc;

pub type AdoscStream = BarStream<Adosc, 4, 1>;

impl Kernel<4, 1> for Adosc {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 4] = ["high", "low", "close", "volume"];
    const OUTPUTS: [&'static str; 1] = ["adosc"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !(FAST_PERIOD_MIN..=FAST_PERIOD_MAX).contains(&params.fast_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "fast_period",
                params.fast_period,
                FAST_PERIOD_MIN,
                FAST_PERIOD_MAX,
            ));
        }
        if !(SLOW_PERIOD_MIN..=SLOW_PERIOD_MAX).contains(&params.slow_period) {
            return Err(TlError::param_out_of_range(
                NAME,
                "slow_period",
                params.slow_period,
                SLOW_PERIOD_MIN,
                SLOW_PERIOD_MAX,
            ));
        }
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        params.fast_period.max(params.slow_period).saturating_sub(1)
    }

    fn state(params: &Params) -> State {
        State {
            line: <ad::Ad as Kernel<4, 1>>::state(&ad::Params),
            // Both averages are seeded with the first value of the A/D line
            // rather than with a mean of the first `period`, because the line
            // starts at the first bar and has no warm-up of its own.
            fast: EmaFromFirst::new(params.fast_period),
            slow: EmaFromFirst::new(params.slow_period),
            seen: 0,
            warm_up: Self::lookback(params),
        }
    }
}
