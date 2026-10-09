use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "sar";

pub const ACCELERATION_DEFAULT: f64 = 0.02;
pub const MAXIMUM_DEFAULT: f64 = 0.2;
pub const STEP_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub acceleration: f64,
    pub maximum: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            acceleration: ACCELERATION_DEFAULT,
            maximum: MAXIMUM_DEFAULT,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Running {
    rising: bool,
    step: f64,
    /// The best price reached since the last flip.
    extreme: f64,
    stop: f64,
}

#[derive(Clone, Debug)]
pub struct State {
    /// The acceleration, already held to the maximum: TA-Lib caps the step
    /// before it starts, so a maximum of zero pins the stop where it began.
    acceleration: f64,
    maximum: f64,
    /// The two bars before the current one, newest last.
    previous: [Option<(f64, f64)>; 2],
    seen: usize,
    running: Option<Running>,
}

impl State {
    /// The lows or highs the stop may not pass: the two bars before this one,
    /// except on the first step. The opening bar is where the stop was placed,
    /// so it does not also hold it back.
    fn held_by(&self) -> impl Iterator<Item = (f64, f64)> + '_ {
        let skip = usize::from(self.seen < 3);
        self.previous.iter().skip(skip).flatten().copied()
    }

    fn start(&self, bar: [f64; 2]) -> Running {
        let (first_high, first_low) = self.previous[1].expect("one bar has been seen");
        // The first direction comes from the larger of the two directional
        // movements over the opening pair, as TA-Lib's does.
        let up = bar[0] - first_high;
        let down = first_low - bar[1];
        let rising = !(down > 0.0 && down > up);
        Running {
            rising,
            step: self.acceleration,
            extreme: if rising { bar[0] } else { bar[1] },
            stop: if rising { first_low } else { first_high },
        }
    }

    fn advance(&self, running: Running, bar: [f64; 2]) -> Running {
        let (high, low) = (bar[0], bar[1]);
        let (previous_high, previous_low) = self.previous[1].expect("a bar has been seen");
        let mut candidate = running.stop + running.step * (running.extreme - running.stop);
        if running.rising {
            for (_, held_low) in self.held_by() {
                candidate = candidate.min(held_low);
            }
            if low <= candidate {
                return Running {
                    rising: false,
                    step: self.acceleration,
                    extreme: low,
                    stop: running.extreme.max(previous_high).max(high),
                };
            }
            Running {
                stop: candidate,
                extreme: running.extreme.max(high),
                step: if high > running.extreme {
                    (running.step + self.acceleration).min(self.maximum)
                } else {
                    running.step
                },
                ..running
            }
        } else {
            for (held_high, _) in self.held_by() {
                candidate = candidate.max(held_high);
            }
            if high >= candidate {
                return Running {
                    rising: true,
                    step: self.acceleration,
                    extreme: high,
                    stop: running.extreme.min(previous_low).min(low),
                };
            }
            Running {
                stop: candidate,
                extreme: running.extreme.min(low),
                step: if low < running.extreme {
                    (running.step + self.acceleration).min(self.maximum)
                } else {
                    running.step
                },
                ..running
            }
        }
    }

    /// What the next bar would read, without changing anything.
    fn next(&self, bar: [f64; 2]) -> Option<(Running, f64)> {
        match self.running {
            None if self.seen == 1 => {
                let started = self.start(bar);
                Some((started, started.stop))
            }
            None => None,
            Some(running) => {
                let next = self.advance(running, bar);
                Some((next, next.stop))
            }
        }
    }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let out = self.next(bar);
        self.previous[0] = self.previous[1];
        self.previous[1] = Some((bar[0], bar[1]));
        self.seen += 1;
        let (next, value) = out?;
        self.running = Some(next);
        Some([value])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        self.next(bar).map(|(_, value)| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sar;

pub type SarStream = BarStream<Sar, 2, 1>;

impl Kernel<2, 1> for Sar {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 1] = ["sar"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        for (name, value) in [
            ("acceleration", params.acceleration),
            ("maximum", params.maximum),
        ] {
            if !value.is_finite() || value < STEP_MIN {
                return Err(TlError::float_param_out_of_range(
                    NAME,
                    name,
                    value,
                    STEP_MIN,
                    f64::INFINITY,
                ));
            }
        }
        Ok(())
    }

    // The direction is read from the first two bars, so the first stop lands
    // on the second of them.
    fn lookback(_params: &Params) -> usize {
        1
    }

    fn state(params: &Params) -> State {
        State {
            acceleration: params.acceleration.min(params.maximum),
            maximum: params.maximum,
            previous: [None, None],
            seen: 0,
            running: None,
        }
    }
}
