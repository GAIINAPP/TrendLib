use crate::core::error::TlError;
use crate::core::kernel::{BarStream, Kernel, Step};

pub const NAME: &str = "sarext";

pub const START_VALUE_DEFAULT: f64 = 0.0;
pub const OFFSET_DEFAULT: f64 = 0.0;
pub const ACCELERATION_DEFAULT: f64 = 0.02;
pub const MAXIMUM_DEFAULT: f64 = 0.2;
pub const STEP_MIN: f64 = 0.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub start_value: f64,
    pub offset_on_reverse: f64,
    pub acceleration_init_long: f64,
    pub acceleration_long: f64,
    pub acceleration_max_long: f64,
    pub acceleration_init_short: f64,
    pub acceleration_short: f64,
    pub acceleration_max_short: f64,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            start_value: START_VALUE_DEFAULT,
            offset_on_reverse: OFFSET_DEFAULT,
            acceleration_init_long: ACCELERATION_DEFAULT,
            acceleration_long: ACCELERATION_DEFAULT,
            acceleration_max_long: MAXIMUM_DEFAULT,
            acceleration_init_short: ACCELERATION_DEFAULT,
            acceleration_short: ACCELERATION_DEFAULT,
            acceleration_max_short: MAXIMUM_DEFAULT,
        }
    }
}

/// One side's step schedule, already held to its own maximum: TA-Lib caps each
/// step before it starts, so a maximum of zero pins the stop where it began.
#[derive(Clone, Copy, Debug)]
struct Schedule {
    start: f64,
    grows_by: f64,
    largest: f64,
}

impl Schedule {
    fn new(start: f64, grows_by: f64, largest: f64) -> Self {
        Self {
            start: start.min(largest),
            grows_by: grows_by.min(largest),
            largest,
        }
    }

    fn grow(&self, step: f64) -> f64 {
        (step + self.grows_by).min(self.largest)
    }
}

#[derive(Clone, Copy, Debug)]
struct Running {
    rising: bool,
    step: f64,
    extreme: f64,
    stop: f64,
}

#[derive(Clone, Debug)]
pub struct State {
    long: Schedule,
    short: Schedule,
    start_value: f64,
    offset: f64,
    /// The bar before the current one, which holds the stop back along with
    /// the current one.
    previous: Option<(f64, f64)>,
    running: Option<Running>,
}

impl State {
    fn start(&self, bar: [f64; 2], first: (f64, f64)) -> Running {
        let (first_high, first_low) = first;
        let rising = if self.start_value == 0.0 {
            // The first direction comes from the larger of the two
            // directional movements over the opening pair.
            let up = bar[0] - first_high;
            let down = first_low - bar[1];
            !(down > 0.0 && down > up)
        } else {
            self.start_value > 0.0
        };
        Running {
            rising,
            step: if rising {
                self.long.start
            } else {
                self.short.start
            },
            extreme: if rising { bar[0] } else { bar[1] },
            stop: if self.start_value == 0.0 {
                if rising { first_low } else { first_high }
            } else {
                self.start_value.abs()
            },
        }
    }

    /// One bar, returning the state to carry and what the row reads.
    fn advance(&self, running: Running, bar: [f64; 2], previous: (f64, f64)) -> (Running, f64) {
        let (high, low) = (bar[0], bar[1]);
        let (previous_high, previous_low) = previous;
        if running.rising {
            if low <= running.stop {
                let mut stop = running.extreme.max(previous_high).max(high);
                // The offset moves the stop itself, not only what is
                // reported, so the next bar advances from the pushed value.
                if self.offset != 0.0 {
                    stop += stop * self.offset;
                }
                let reported = -stop;
                let step = self.short.start;
                let extreme = low;
                let carried = (stop + step * (extreme - stop))
                    .max(previous_high)
                    .max(high);
                return (
                    Running {
                        rising: false,
                        step,
                        extreme,
                        stop: carried,
                    },
                    reported,
                );
            }
            let reported = running.stop;
            let (extreme, step) = if high > running.extreme {
                (high, self.long.grow(running.step))
            } else {
                (running.extreme, running.step)
            };
            let carried = (running.stop + step * (extreme - running.stop))
                .min(previous_low)
                .min(low);
            (
                Running {
                    rising: true,
                    step,
                    extreme,
                    stop: carried,
                },
                reported,
            )
        } else {
            if high >= running.stop {
                let mut stop = running.extreme.min(previous_low).min(low);
                if self.offset != 0.0 {
                    stop -= stop * self.offset;
                }
                let reported = stop;
                let step = self.long.start;
                let extreme = high;
                let carried = (stop + step * (extreme - stop)).min(previous_low).min(low);
                return (
                    Running {
                        rising: true,
                        step,
                        extreme,
                        stop: carried,
                    },
                    reported,
                );
            }
            let reported = -running.stop;
            let (extreme, step) = if low < running.extreme {
                (low, self.short.grow(running.step))
            } else {
                (running.extreme, running.step)
            };
            let carried = (running.stop + step * (extreme - running.stop))
                .max(previous_high)
                .max(high);
            (
                Running {
                    rising: false,
                    step,
                    extreme,
                    stop: carried,
                },
                reported,
            )
        }
    }

    fn next(&self, bar: [f64; 2]) -> Option<(Running, f64)> {
        let first = self.previous?;
        let running = match self.running {
            Some(running) => running,
            // On the first bar the stop has only itself to be held back by,
            // which is TA-Lib's "cheat" of setting the previous bar to this
            // one before the first step.
            None => return Some(self.advance(self.start(bar, first), bar, (bar[0], bar[1]))),
        };
        Some(self.advance(running, bar, first))
    }
}

impl Step<2, 1> for State {
    fn push(&mut self, bar: [f64; 2]) -> Option<[f64; 1]> {
        let out = self.next(bar);
        self.previous = Some((bar[0], bar[1]));
        let (next, value) = out?;
        self.running = Some(next);
        Some([value])
    }

    fn preview(&self, bar: [f64; 2]) -> Option<[f64; 1]> {
        self.next(bar).map(|(_, value)| [value])
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Sarext;

pub type SarextStream = BarStream<Sarext, 2, 1>;

impl Kernel<2, 1> for Sarext {
    const NAME: &'static str = NAME;
    const INPUTS: [&'static str; 2] = ["high", "low"];
    const OUTPUTS: [&'static str; 1] = ["sarext"];

    type Params = Params;
    type State = State;

    fn validate(params: &Params) -> Result<(), TlError> {
        if !params.start_value.is_finite() {
            return Err(TlError::float_param_out_of_range(
                NAME,
                "start_value",
                params.start_value,
                f64::NEG_INFINITY,
                f64::INFINITY,
            ));
        }
        for (name, value) in [
            ("offset_on_reverse", params.offset_on_reverse),
            ("acceleration_init_long", params.acceleration_init_long),
            ("acceleration_long", params.acceleration_long),
            ("acceleration_max_long", params.acceleration_max_long),
            ("acceleration_init_short", params.acceleration_init_short),
            ("acceleration_short", params.acceleration_short),
            ("acceleration_max_short", params.acceleration_max_short),
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

    fn lookback(_params: &Params) -> usize {
        1
    }

    fn state(params: &Params) -> State {
        State {
            long: Schedule::new(
                params.acceleration_init_long,
                params.acceleration_long,
                params.acceleration_max_long,
            ),
            short: Schedule::new(
                params.acceleration_init_short,
                params.acceleration_short,
                params.acceleration_max_short,
            ),
            start_value: params.start_value,
            offset: params.offset_on_reverse,
            previous: None,
            running: None,
        }
    }
}
