use crate::core::chart::whole;
use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::indicators::alligator;

pub const NAME: &str = "gator";

pub const JAW_PERIOD_DEFAULT: usize = 13;
pub const JAW_PERIOD_MIN: usize = 1;
pub const JAW_PERIOD_MAX: usize = 100_000;
pub const JAW_SHIFT_DEFAULT: usize = 8;
pub const JAW_SHIFT_MIN: usize = 1;
pub const JAW_SHIFT_MAX: usize = 100_000;
pub const TEETH_PERIOD_DEFAULT: usize = 8;
pub const TEETH_PERIOD_MIN: usize = 1;
pub const TEETH_PERIOD_MAX: usize = 100_000;
pub const TEETH_SHIFT_DEFAULT: usize = 5;
pub const TEETH_SHIFT_MIN: usize = 1;
pub const TEETH_SHIFT_MAX: usize = 100_000;
pub const LIPS_PERIOD_DEFAULT: usize = 5;
pub const LIPS_PERIOD_MIN: usize = 1;
pub const LIPS_PERIOD_MAX: usize = 100_000;
pub const LIPS_SHIFT_DEFAULT: usize = 3;
pub const LIPS_SHIFT_MIN: usize = 1;
pub const LIPS_SHIFT_MAX: usize = 100_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Params {
    pub jaw_period: usize,
    pub jaw_shift: usize,
    pub teeth_period: usize,
    pub teeth_shift: usize,
    pub lips_period: usize,
    pub lips_shift: usize,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            jaw_period: JAW_PERIOD_DEFAULT,
            jaw_shift: JAW_SHIFT_DEFAULT,
            teeth_period: TEETH_PERIOD_DEFAULT,
            teeth_shift: TEETH_SHIFT_DEFAULT,
            lips_period: LIPS_PERIOD_DEFAULT,
            lips_shift: LIPS_SHIFT_DEFAULT,
        }
    }
}

#[derive(Clone, Debug)]
pub struct State(alligator::State);

fn base(params: &Params) -> alligator::Params {
    alligator::Params {
        jaw_period: params.jaw_period,
        jaw_shift: params.jaw_shift,
        teeth_period: params.teeth_period,
        teeth_shift: params.teeth_shift,
        lips_period: params.lips_period,
        lips_shift: params.lips_shift,
    }
}

fn spread([jaw, teeth, lips]: [f64; 3]) -> [f64; 2] {
    [(jaw - teeth).abs(), -(teeth - lips).abs()]
}

impl Step<2, 2> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 2]> {
        self.0.advance(bar).map(spread)
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 2]> {
        self.0.clone().advance(bar).map(spread)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Gator;

pub type GatorStream = BarStream<Gator, 2, 2>;

impl Kernel<2, 2> for Gator {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 2] = ["gator_upper", "gator_lower"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        whole(
            NAME,
            "jaw_period",
            params.jaw_period,
            JAW_PERIOD_MIN,
            JAW_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "jaw_shift",
            params.jaw_shift,
            JAW_SHIFT_MIN,
            JAW_SHIFT_MAX,
        )?;
        whole(
            NAME,
            "teeth_period",
            params.teeth_period,
            TEETH_PERIOD_MIN,
            TEETH_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "teeth_shift",
            params.teeth_shift,
            TEETH_SHIFT_MIN,
            TEETH_SHIFT_MAX,
        )?;
        whole(
            NAME,
            "lips_period",
            params.lips_period,
            LIPS_PERIOD_MIN,
            LIPS_PERIOD_MAX,
        )?;
        whole(
            NAME,
            "lips_shift",
            params.lips_shift,
            LIPS_SHIFT_MIN,
            LIPS_SHIFT_MAX,
        )?;
        Ok(())
    }

    fn lookback(params: &Params) -> usize {
        alligator::Alligator::lookback(&base(params))
    }

    fn state(params: &Params) -> State {
        State(alligator::State::new(&base(params)))
    }
}
