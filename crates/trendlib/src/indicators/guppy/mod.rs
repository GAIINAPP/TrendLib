use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};
use crate::core::math::Ema;

pub const NAME: &str = "guppy";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

const PERIODS: [usize; 12] = [3, 5, 8, 10, 12, 15, 30, 35, 40, 45, 50, 60];

#[derive(Clone, Debug)]
pub struct State {
    averages: [Ema; 12],
}

impl Step<1, 12> for State {
    fn push(&mut self, bar: [f64; 1]) -> Option<[f64; 12]> {
        let lines: [Option<f64>; 12] = std::array::from_fn(|i| self.averages[i].push(bar[0]));
        // The longest average is the last to start; once it has, all have.
        lines[11].map(|_| lines.map(|line| line.unwrap_or(f64::NAN)))
    }

    fn preview(&self, bar: [f64; 1]) -> Option<[f64; 12]> {
        let lines: [Option<f64>; 12] = std::array::from_fn(|i| self.averages[i].preview(bar[0]));
        lines[11].map(|_| lines.map(|line| line.unwrap_or(f64::NAN)))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Guppy;

pub type GuppyStream = BarStream<Guppy, 1, 12>;

impl Kernel<1, 12> for Guppy {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 1] = ["source"];
    const OUTPUTS: [&'static str; 12] = [
        "guppy_short_3",
        "guppy_short_5",
        "guppy_short_8",
        "guppy_short_10",
        "guppy_short_12",
        "guppy_short_15",
        "guppy_long_30",
        "guppy_long_35",
        "guppy_long_40",
        "guppy_long_45",
        "guppy_long_50",
        "guppy_long_60",
    ];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        PERIODS[11] - 1
    }

    fn state(_params: &Params) -> State {
        State {
            averages: PERIODS.map(Ema::new),
        }
    }
}
