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
/// average, then nine more that TA-Lib runs through it without output.
const PRIMED: usize = 12;

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
        (NEAR * h[0] + FAR * h[2] - FAR * h[4] - NEAR * h[6]) * adjusted
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

#[derive(Clone, Debug, Default)]
pub struct Hilbert {
    /// The four-bar weighted average of the input, newest first.
    smoothed: [f64; 4],
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
    /// The input itself, newest first, for the four-bar weighted average.
    raw: [f64; 4],
}

impl Hilbert {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bars consumed before the transform starts running.
    pub fn primed() -> usize {
        PRIMED
    }

    /// The four-bar weighted average TA-Lib smooths the input with.
    fn smooth(&self) -> f64 {
        (4.0 * self.raw[0] + 3.0 * self.raw[1] + 2.0 * self.raw[2] + self.raw[3]) / 10.0
    }

    pub fn push(&mut self, value: f64) -> Option<Phase> {
        self.raw.rotate_right(1);
        self.raw[0] = value;
        self.seen += 1;
        self.smoothed.rotate_right(1);
        self.smoothed[0] = if self.seen >= 4 { self.smooth() } else { 0.0 };
        if self.seen <= PRIMED {
            return None;
        }

        let adjusted = 0.075 * self.period + 0.54;
        let detrender = self.detrender.push(self.smoothed[0], adjusted);
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
