//! Shared row-scroll state for scrollable module panels.
//!
//! Owns the offset number and the math around it: how far the content may
//! scroll, what a wheel step does, and how pinned top/bottom blocks shrink
//! the scrollable middle. Modules keep ownership of their content rows and
//! their draw/crop logic; this type only answers "which row is first?".

/// Row offset of a scrollable panel: how many content rows are scrolled
/// past from the top; 0 shows the top of the content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollState {
    offset: usize,
}

impl ScrollState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Rows scrolled past from the top of the content.
    pub fn offset(self) -> usize {
        self.offset
    }

    /// Largest valid offset for `total_rows` shown in a `viewport_rows`
    /// tall window. Saturates at 0 so short content simply never scrolls.
    pub fn max_offset(total_rows: usize, viewport_rows: usize) -> usize {
        total_rows.saturating_sub(viewport_rows)
    }

    /// Middle rows available for scrolling content once pinned top and
    /// bottom blocks take their share of the viewport.
    pub fn available_rows(viewport_rows: usize, pinned_top: usize, pinned_bottom: usize) -> usize {
        viewport_rows
            .saturating_sub(pinned_top)
            .saturating_sub(pinned_bottom)
    }

    /// One wheel step moves one row: wheel up (positive `delta_y`) toward
    /// the top, wheel down toward the bottom, clamped to `[0, max]`.
    /// Returns whether the offset moved, so modules can decide whether to
    /// consume the wheel event.
    pub fn wheel(&mut self, delta_y: f32, max: usize) -> bool {
        let next = if delta_y > 0.0 {
            self.offset.saturating_sub(1)
        } else if delta_y < 0.0 {
            (self.offset + 1).min(max)
        } else {
            return false;
        };
        let moved = next != self.offset;
        self.offset = next;
        moved
    }

    /// Jump to the very top of the content.
    pub fn to_top(&mut self) {
        self.offset = 0;
    }

    /// Jump to the very bottom of the content for the given max offset.
    pub fn to_end(&mut self, max: usize) {
        self.offset = max;
    }
}

#[cfg(test)]
mod tests {
    use super::ScrollState;

    #[test]
    fn max_offset_saturates_at_zero_for_short_content() {
        assert_eq!(ScrollState::max_offset(5, 10), 0);
        assert_eq!(ScrollState::max_offset(10, 10), 0);
        assert_eq!(ScrollState::max_offset(15, 10), 5);
    }

    #[test]
    fn available_rows_shrinks_by_both_pinned_blocks() {
        assert_eq!(ScrollState::available_rows(20, 2, 0), 18);
        assert_eq!(ScrollState::available_rows(20, 2, 3), 15);
        // Pinned blocks larger than the viewport leave nothing scrollable.
        assert_eq!(ScrollState::available_rows(4, 3, 3), 0);
    }

    #[test]
    fn wheel_steps_one_row_and_clamps_at_both_edges() {
        let mut scroll = ScrollState::new();
        // Wheel up at the top stays put and reports no movement.
        assert!(!scroll.wheel(1.0, 5));
        assert_eq!(scroll.offset(), 0);
        // Wheel down steps toward the bottom and clamps at max.
        assert!(scroll.wheel(-1.0, 5));
        assert!(scroll.wheel(-1.0, 5));
        assert_eq!(scroll.offset(), 2);
        for _ in 0..10 {
            scroll.wheel(-1.0, 5);
        }
        assert_eq!(scroll.offset(), 5);
        // Wheel up walks back toward the top.
        assert!(scroll.wheel(1.0, 5));
        assert_eq!(scroll.offset(), 4);
        // Zero delta is a no-op.
        assert!(!scroll.wheel(0.0, 5));
        assert_eq!(scroll.offset(), 4);
    }

    #[test]
    fn jump_helpers_land_exactly_on_the_edges() {
        let mut scroll = ScrollState::new();
        scroll.wheel(-1.0, 5);
        scroll.to_end(5);
        assert_eq!(scroll.offset(), 5);
        scroll.to_top();
        assert_eq!(scroll.offset(), 0);
    }

    #[test]
    fn wheel_into_empty_max_stays_at_top() {
        let mut scroll = ScrollState::new();
        assert!(!scroll.wheel(-1.0, 0));
        assert_eq!(scroll.offset(), 0);
    }
}
