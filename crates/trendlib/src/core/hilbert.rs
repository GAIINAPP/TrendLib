//! The machinery the cycle indicators share.
//!
//! John Ehlers' construction treats the series as a signal, splits it into a
//! part in phase with the dominant cycle and a part a quarter turn ahead of it,
//! and reads the cycle's length from how fast the phase turns. Every indicator
//! in the `cycle` group is one reading of the state below.
//!
//! The arithmetic follows TA-Lib's, including where it starts: the transform's
//! buffers begin empty, so the bars before it starts running count as zero
//! rather than as the values they actually had.

/// The two weights of Ehlers' four-tap transform.
const NEAR: f64 = 0.0962;
const FAR: f64 = 0.5769;

/// Bars consumed before the transform starts: three to fill the weighted
/// average, then the ones TA-Lib runs through it without output. It runs nine
/// of them for the readings that warm up in 32 bars and thirty-four for those
/// that take 63, which puts the transform's own start twenty-five bars later
/// and is the difference between reproducing the oracle and missing it by one
/// part in forty.
pub const PRIMED_EARLY: usize = 12;
pub const PRIMED_LATE: usize = 37;

/// How far back the cycle readings can look: the dominant cycle is held to
/// fifty bars, so nothing ever reaches further.
pub const WINDOW: usize = 50;

/// The last fifty of something, newest first.
#[derive(Clone, Copy, Debug)]
pub struct Recent {
    values: [f64; WINDOW],
}

impl Default for Recent {
    fn default() -> Self {
        Self {
            values: [0.0; WINDOW],
        }
    }
}

impl Recent {
    pub fn push(&mut self, value: f64) {
        self.values.rotate_right(1);
        self.values[0] = value;
    }

    /// The value `back` bars ago, or zero for a bar this has not seen.
    pub fn back(&self, back: usize) -> f64 {
        self.values.get(back).copied().unwrap_or(0.0)
    }

    /// The mean of the newest `count` values, as TA-Lib takes it: a fixed
    /// fifty-step loop that adds only the ones inside the window.
    pub fn mean(&self, count: usize) -> f64 {
        if count == 0 {
            return 0.0;
        }
        let total: f64 = (0..WINDOW)
            .filter(|k| *k < count)
            .map(|k| self.values[k])
            .sum();
        total / count as f64
    }
}

/// A four-tap Hilbert transform over every second bar.
///
/// The taps are the current value and the ones two, four and six bars back,
/// which is why it keeps its own history rather than reading one.
#[derive(Clone, Copy, Debug, Default)]
struct Transform {
    history: [f64; 7],
}

impl Transform {
    fn push(&mut self, value: f64, adjusted: f64) -> f64 {
        self.history.rotate_right(1);
        self.history[0] = value;
        let h = &self.history;
        // The four taps are added in TA-Lib's order, which is not the one the
        // formula is written in. On a stretch where the input never moves the
        // four terms cancel to a residue a few ULP wide, and whether that
        // residue is exactly zero decides a branch in `mama`: the two orders
        // disagree by 28 on a %K line that sat at 100 for five bars.
        ((NEAR * h[0] - NEAR * h[6]) - FAR * h[4] + FAR * h[2]) * adjusted
    }
}

/// Everything the cycle indicators read, as of the newest bar.
#[derive(Clone, Copy, Debug, Default)]
pub struct Phase {
    /// The part of the signal in phase with the cycle.
    pub in_phase: f64,
    /// The part a quarter turn ahead of it.
    pub quadrature: f64,
    /// The dominant cycle length, smoothed.
    pub smooth_period: f64,
    /// The same before the final smoothing, which the period's own recursion
    /// carries forward.
    pub period: f64,
}

/// The four-bar weighted average, kept the way TA-Lib keeps it.
///
/// It carries a running weighted sum rather than re-adding four terms each
/// bar. The two agree to within a few ULP, and a few ULP is the difference
/// between a flat stretch detrending to exactly zero and to a residue, which
/// `mama` branches on.
#[derive(Clone, Copy, Debug, Default)]
struct Weighted {
    sum: f64,
    sub: f64,
    trailing: f64,
    seen: usize,
    /// The last four inputs, newest first, so the one about to leave the
    /// window can be read.
    inputs: [f64; 4],
}

impl Weighted {
    fn push(&mut self, value: f64) -> f64 {
        self.inputs.rotate_right(1);
        self.inputs[0] = value;
        self.seen += 1;
        match self.seen {
            1 => {
                self.sub = value;
                self.sum = value;
                0.0
            }
            2 => {
                self.sub += value;
                self.sum += value * 2.0;
                0.0
            }
            3 => {
                self.sub += value;
                self.sum += value * 3.0;
                0.0
            }
            _ => {
                self.sub += value;
                self.sub -= self.trailing;
                self.sum += value * 4.0;
                // The bar that will leave the window next time is three back
                // from this one; it is picked up after the subtraction, so the
                // first weighted value has nothing to shed.
                self.trailing = self.inputs[3];
                let smoothed = self.sum * 0.1;
                self.sum -= self.sub;
                smoothed
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Hilbert {
    /// The four-bar weighted average of the input.
    weighted: Weighted,
    detrender: Transform,
    quadrature: Transform,
    in_phase_tap: Transform,
    quadrature_tap: Transform,
    /// The last four detrender values, so the in-phase part can be read from
    /// three bars back.
    detrended: [f64; 4],
    previous_in_phase: f64,
    previous_quadrature: f64,
    real: f64,
    imaginary: f64,
    period: f64,
    smooth_period: f64,
    seen: usize,
    /// The newest weighted average, which the transform is fed.
    smoothed: f64,
    /// Where the transform starts running.
    primed: usize,
    /// The weighted averages it has seen, which the phase readings sum over.
    history: Recent,
}

impl Hilbert {
    pub fn new(primed: usize) -> Self {
        Self {
            primed,
            ..Self::default()
        }
    }

    /// The weighted averages the transform has seen, newest first. Bars before
    /// it started count as zero, which is how TA-Lib's buffers begin.
    pub fn history(&self) -> &Recent {
        &self.history
    }

    pub fn push(&mut self, value: f64) -> Option<Phase> {
        self.smoothed = self.weighted.push(value);
        self.seen += 1;
        if self.seen <= self.primed {
            return None;
        }
        self.history.push(self.smoothed);

        let adjusted = 0.075 * self.period + 0.54;
        let detrender = self.detrender.push(self.smoothed, adjusted);
        let quadrature = self.quadrature.push(detrender, adjusted);
        self.detrended.rotate_right(1);
        self.detrended[0] = detrender;
        let in_phase = self.detrended[3];
        let lead_in_phase = self.in_phase_tap.push(in_phase, adjusted);
        let lead_quadrature = self.quadrature_tap.push(quadrature, adjusted);

        let smoothed_in_phase = 0.2 * (in_phase - lead_quadrature) + 0.8 * self.previous_in_phase;
        let smoothed_quadrature =
            0.2 * (quadrature + lead_in_phase) + 0.8 * self.previous_quadrature;
        let real = smoothed_in_phase * self.previous_in_phase
            + smoothed_quadrature * self.previous_quadrature;
        let imaginary = smoothed_in_phase * self.previous_quadrature
            - smoothed_quadrature * self.previous_in_phase;
        self.previous_in_phase = smoothed_in_phase;
        self.previous_quadrature = smoothed_quadrature;
        self.real = 0.2 * real + 0.8 * self.real;
        self.imaginary = 0.2 * imaginary + 0.8 * self.imaginary;

        // A full turn divided by how far the phase moved is the cycle's
        // length. The clamps keep it from changing by more than half in a bar
        // and hold it between six and fifty.
        let mut next = self.period;
        if self.imaginary != 0.0 && self.real != 0.0 {
            next = 360.0 / ((self.imaginary / self.real).atan() * (180.0 / std::f64::consts::PI));
        }
        next = next.min(1.5 * self.period).max(0.67 * self.period);
        next = next.clamp(6.0, 50.0);
        self.period = 0.2 * next + 0.8 * self.period;
        self.smooth_period = 0.33 * self.period + 0.67 * self.smooth_period;

        Some(Phase {
            in_phase,
            quadrature,
            smooth_period: self.smooth_period,
            period: self.period,
        })
    }

    /// What the next bar would read, without changing anything.
    pub fn preview(&self, value: f64) -> Option<Phase> {
        let mut forked = self.clone();
        forked.push(value)
    }
}

/// Where the dominant cycle currently sits in its turn, in degrees.
///
/// The phase is carried from bar to bar: when the imaginary part vanishes the
/// reading is nudged rather than recomputed, so the previous value is part of
/// the next one.
#[derive(Clone, Copy, Debug, Default)]
pub struct DominantPhase {
    degrees: f64,
}

impl DominantPhase {
    /// One bar of the phase, given the transform's state. Returns degrees.
    pub fn push(&mut self, cycle: &Hilbert, smooth_period: f64) -> f64 {
        self.degrees = Self::next(self.degrees, cycle, smooth_period);
        self.degrees
    }

    /// What the next bar would read, without changing anything.
    pub fn preview(&self, cycle: &Hilbert, smooth_period: f64) -> f64 {
        Self::next(self.degrees, cycle, smooth_period)
    }

    fn next(mut degrees: f64, cycle: &Hilbert, smooth_period: f64) -> f64 {
        let count = (smooth_period + 0.5) as usize;
        let mut real = 0.0;
        let mut imaginary = 0.0;
        for step in 0..count {
            let angle = step as f64 * std::f64::consts::TAU / count as f64;
            let value = cycle.history().back(step);
            real += angle.sin() * value;
            imaginary += angle.cos() * value;
        }
        if imaginary.abs() > 0.0 {
            degrees = (real / imaginary).atan() * (180.0 / std::f64::consts::PI);
        } else if imaginary.abs() <= 0.01 {
            // Nothing to take an angle of, so the reading is nudged a quarter
            // turn in whichever direction the real part points.
            if real < 0.0 {
                degrees -= 90.0;
            } else if real > 0.0 {
                degrees += 90.0;
            }
        }
        degrees += 90.0;
        // One bar of lag from the weighted average the transform smooths with.
        degrees += 360.0 / smooth_period;
        if imaginary < 0.0 {
            degrees += 180.0;
        }
        if degrees > 315.0 {
            degrees -= 360.0;
        }
        degrees
    }
}
