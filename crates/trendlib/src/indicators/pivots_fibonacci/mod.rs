use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "pivots_fibonacci";

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Params;

#[derive(Clone, Debug)]
pub struct State {
    previous: Option<[f64; 3]>,
}

impl State {
    fn levels(previous: [f64; 3]) -> [f64; 9] {
        let [high, low, close] = previous;
        let pivot = (high + low + close) / 3.0;
        let range = high - low;
        [
            pivot,
            pivot - range * 0.382,
            pivot - range * 0.618,
            pivot - range * 1.0,
            pivot - range * 1.382,
            pivot + range * 0.382,
            pivot + range * 0.618,
            pivot + range * 1.0,
            pivot + range * 1.382,
        ]
    }
}

impl Step<3, 9> for State {
    fn push(&mut self, bar: [f64; 3]) -> Option<[f64; 9]> {
        let levels = self.previous.map(Self::levels);
        self.previous = Some(bar);
        levels
    }

    fn preview(&self, _bar: [f64; 3]) -> Option<[f64; 9]> {
        self.previous.map(Self::levels)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PivotsFibonacci;

pub type PivotsFibonacciStream = BarStream<PivotsFibonacci, 3, 9>;

impl Kernel<3, 9> for PivotsFibonacci {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 3] = ["high", "low", "close"];
    const OUTPUTS: [&'static str; 9] = [
        "fibonacci_pp",
        "fibonacci_s1",
        "fibonacci_s2",
        "fibonacci_s3",
        "fibonacci_s4",
        "fibonacci_r1",
        "fibonacci_r2",
        "fibonacci_r3",
        "fibonacci_r4",
    ];

    type Params = Params;
    type State = State;

    fn validate(_params: &Params) -> Result<(), TlError> {
        Ok(())
    }

    fn lookback(_params: &Params) -> usize {
        1
    }

    fn state(_params: &Params) -> State {
        State { previous: None }
    }
}
