//! The host's clock: the dilation, the pause, and the conversion of real time
//! into simulated time.
//!
//! **This is the host, not the simulation.** The dilation lives here because the
//! simulation must never be told which one is in force — that is what makes
//! dilation independence hold. A campaign played at 1x and at 10,000x is the same
//! campaign, and it can only be that because the simulation sees nothing but a
//! duration.
//!
//! See `docs/SIMULATION.md` §2.3 and §6.3.

use std::time::{Duration as RealDuration, Instant};

use domain::time::{Duration, TimeScale};

/// How long the host waits for a key before it redraws, in milliseconds.
///
/// This bounds the frame rate when nothing is happening. It is deliberately
/// short enough that the clock stays smooth, and long enough that a terminal is
/// not busy-polled.
pub const FRAME_MILLIS: u64 = 16;

/// The host's clock.
///
/// It owns the dilation and the pause, and it turns real elapsed time into
/// simulated elapsed time. It does **not** own the simulated instant — that
/// belongs to the simulation.
#[derive(Debug)]
pub struct Clock {
    scale: TimeScale,
    paused: bool,
    last_real: Instant,
}

impl Clock {
    /// A clock running at the given dilation.
    #[must_use]
    pub fn new(scale: TimeScale) -> Self {
        Self {
            scale,
            paused: false,
            last_real: Instant::now(),
        }
    }

    /// The current dilation.
    #[must_use]
    pub const fn scale(&self) -> TimeScale {
        self.scale
    }

    /// Sets the dilation.
    pub fn set_scale(&mut self, scale: TimeScale) {
        self.scale = scale;
    }

    /// One dilation step faster.
    pub fn faster(&mut self) {
        self.scale = self.scale.faster();
    }

    /// One dilation step slower.
    pub fn slower(&mut self) {
        self.scale = self.scale.slower();
    }

    /// Whether the clock is stopped.
    #[must_use]
    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    /// Stops or starts the clock.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// The simulated time that has passed since the last call.
    ///
    /// Returns nothing while the clock is paused. The real elapsed time is
    /// consumed either way, so **unpausing does not release a backlog** — the
    /// campaign does not jump forward by however long it was stopped.
    pub fn tick(&mut self) -> Duration {
        let now = Instant::now();
        let real = now.saturating_duration_since(self.last_real);
        self.last_real = now;
        self.simulated_for(real)
    }

    /// The simulated time corresponding to a given amount of real time.
    ///
    /// This is the whole of the dilation, and it is pure so that it can be
    /// tested without a clock.
    #[must_use]
    pub fn simulated_for(&self, real: RealDuration) -> Duration {
        if self.paused {
            return Duration::ZERO;
        }
        let millis = real.as_millis();
        let scaled = millis.saturating_mul(u128::from(self.scale.multiplier()));
        Duration::from_millis(u64::try_from(scaled).unwrap_or(u64::MAX))
    }

    /// A short label for the top bar.
    #[must_use]
    pub fn label(&self) -> String {
        if self.paused {
            "paused".to_owned()
        } else {
            format!("{}x", self.scale.multiplier())
        }
    }
}

impl Default for Clock {
    fn default() -> Self {
        Self::new(TimeScale::Sixty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixty_x_is_a_minute_a_second() {
        // **The base rate: one second in life is one minute in game.**
        let clock = Clock::new(TimeScale::Sixty);
        let real = RealDuration::from_secs(3);
        assert_eq!(clock.simulated_for(real), Duration::from_secs(180));
    }

    #[test]
    fn a_sixty_thousand_x_is_sixty_thousand_times_faster() {
        let clock = Clock::new(TimeScale::SixtyThousand);
        let real = RealDuration::from_secs(3);
        assert_eq!(clock.simulated_for(real), Duration::from_secs(180_000));
    }

    #[test]
    fn every_dilation_scales_the_same_way() {
        for scale in TimeScale::ALL {
            let clock = Clock::new(scale);
            let real = RealDuration::from_millis(1_000);
            let expected = scale.multiplier() * 1_000;
            assert_eq!(clock.simulated_for(real), Duration::from_millis(expected));
        }
    }

    #[test]
    fn a_paused_clock_simulates_nothing() {
        let mut clock = Clock::new(TimeScale::SixHundredThousand);
        clock.toggle_pause();
        assert!(clock.is_paused());
        let real = RealDuration::from_secs(60);
        assert!(clock.simulated_for(real).is_zero());
    }

    #[test]
    fn the_dilation_steps_both_ways() {
        let mut clock = Clock::new(TimeScale::SixThousand);
        clock.faster();
        assert_eq!(clock.scale(), TimeScale::ThirtyThousand);
        clock.slower();
        assert_eq!(clock.scale(), TimeScale::SixThousand);
    }

    #[test]
    fn the_dilation_never_leaves_its_range() {
        let mut clock = Clock::new(TimeScale::Sixty);
        for _ in 0..20 {
            clock.slower();
        }
        assert_eq!(clock.scale(), TimeScale::Sixty);
        for _ in 0..20 {
            clock.faster();
        }
        assert_eq!(clock.scale(), TimeScale::SixHundredThousand);
    }

    #[test]
    fn the_label_says_whether_the_clock_is_stopped() {
        let mut clock = Clock::new(TimeScale::SixtyThousand);
        assert_eq!(clock.label(), "60000x");
        clock.toggle_pause();
        assert_eq!(clock.label(), "paused");
    }

    #[test]
    fn the_clock_starts_at_the_base_rate() {
        // **One second in life is one minute in game**, and that is where the
        // campaign starts.
        assert_eq!(Clock::default().scale(), TimeScale::Sixty);
    }
}
