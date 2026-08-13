use std::time::Duration;

pub const FALLBACK_BREATH_TICK_DURATION: Duration = Duration::from_millis(250);
pub const FALLBACK_BREATH_WRAP_MODULUS: i32 = 1_048_576;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FallbackBreathClock {
    breath: i32,
}

impl Default for FallbackBreathClock {
    fn default() -> Self {
        Self { breath: 0 }
    }
}

impl FallbackBreathClock {
    pub fn new(initial_breath: i32) -> Self {
        Self {
            breath: wrap_breath(initial_breath),
        }
    }

    pub const fn current(&self) -> i32 {
        self.breath
    }

    pub fn tick(&mut self) -> i32 {
        self.breath = wrap_breath(self.breath + 1);
        self.breath
    }
}

const fn wrap_breath(value: i32) -> i32 {
    value.rem_euclid(FALLBACK_BREATH_WRAP_MODULUS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_breath_clock_starts_wrapped() {
        let clock = FallbackBreathClock::new(FALLBACK_BREATH_WRAP_MODULUS + 7);
        assert_eq!(clock.current(), 7);
    }

    #[test]
    fn fallback_breath_clock_ticks_by_one_every_step() {
        let mut clock = FallbackBreathClock::default();
        assert_eq!(clock.tick(), 1);
        assert_eq!(clock.tick(), 2);
        assert_eq!(clock.tick(), 3);
    }

    #[test]
    fn fallback_breath_clock_wraps_safely_at_the_high_modulus() {
        let mut clock = FallbackBreathClock::new(FALLBACK_BREATH_WRAP_MODULUS - 1);
        assert_eq!(clock.tick(), 0);
        assert_eq!(clock.tick(), 1);
    }

    #[test]
    fn fallback_breath_tick_duration_matches_the_quarter_second_contract() {
        assert_eq!(FALLBACK_BREATH_TICK_DURATION, Duration::from_millis(250));
    }
}
