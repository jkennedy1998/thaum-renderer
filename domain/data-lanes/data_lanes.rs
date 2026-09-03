#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DataLanes {
    breath: Option<i32>,
}

impl DataLanes {
    pub const fn new() -> Self {
        Self { breath: None }
    }

    pub const fn with_breath(breath: i32) -> Self {
        Self {
            breath: Some(breath),
        }
    }

    pub const fn breath(&self) -> Option<i32> {
        self.breath
    }

    pub const fn has_breath(&self) -> bool {
        self.breath.is_some()
    }

    pub fn set_breath(&mut self, breath: i32) {
        self.breath = Some(breath);
    }

    pub fn set_fallback_breath_if_unset(&mut self, breath: i32) {
        if self.breath.is_none() {
            self.breath = Some(breath);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_data_lanes_start_without_breath() {
        let lanes = DataLanes::default();
        assert_eq!(lanes.breath(), None);
        assert!(!lanes.has_breath());
    }

    #[test]
    fn with_breath_marks_the_lane_as_present() {
        let lanes = DataLanes::with_breath(42);
        assert_eq!(lanes.breath(), Some(42));
        assert!(lanes.has_breath());
    }

    #[test]
    fn fallback_breath_only_fills_an_unset_lane() {
        let mut lanes = DataLanes::default();
        lanes.set_fallback_breath_if_unset(7);
        assert_eq!(lanes.breath(), Some(7));

        lanes.set_breath(99);
        lanes.set_fallback_breath_if_unset(12);
        assert_eq!(lanes.breath(), Some(99));
    }
}
