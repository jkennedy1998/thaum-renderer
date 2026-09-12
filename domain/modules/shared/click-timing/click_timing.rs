//! Shared double-click detection: the one helper for deciding whether two
//! pointer clicks count as a double-click, keyed on an equality-comparable
//! "click subject" the caller picks.
//!
//! Modules kept re-rolling this as hand-rolled `Option<Recent…Click>` structs
//! with a local `Duration::from_millis(350)` window and an equality check per
//! field (the painter layers panel grew two before this seam existed). The
//! detector generalizes it: the caller describes what was clicked with any
//! `Eq + Clone` subject — a layer id, a `(button, block-id, piece)` tuple, a
//! cell coordinate — and the detector owns the window timing and the
//! same-subject comparison.
//!
//! The detector is pure state + timing; it draws nothing, owns no rect, and
//! reads no clock of its own. Callers pass `Instant::now()`, which keeps it
//! testable without any sleeping.

use std::time::{Duration, Instant};

/// The canonical double-click window, matching the old modules'
/// `DOUBLE_CLICK_WINDOW` constant (350 ms between clicks).
pub const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(350);

#[derive(Debug, Clone)]
struct RecentClick<K> {
    subject: K,
    at: Instant,
}

/// One double-click detector per click surface. Generic over the click
/// subject; `K = ()` degenerates to a plain windowed double-click.
#[derive(Debug, Clone)]
pub struct DoubleClick<K: Eq + Clone> {
    window: Duration,
    recent: Option<RecentClick<K>>,
}

impl<K: Eq + Clone> DoubleClick<K> {
    /// A detector on the canonical 350 ms window.
    pub fn new() -> Self {
        Self {
            window: DOUBLE_CLICK_WINDOW,
            recent: None,
        }
    }

    /// A detector on a custom window, for surfaces with a different feel.
    pub fn with_window(window: Duration) -> Self {
        Self { window, recent: None }
    }

    /// Record one click on `subject` and report whether it completes a
    /// double-click: the previous click was on an equal subject inside the
    /// window. Every click becomes the new "recent" click, so a triple-click
    /// reads as double-click, then double-click.
    pub fn note_click(&mut self, subject: K, now: Instant) -> bool {
        let is_double_click = self
            .recent
            .as_ref()
            .map(|recent| recent.subject == subject && now.duration_since(recent.at) <= self.window)
            .unwrap_or(false);
        self.recent = Some(RecentClick { subject, at: now });
        is_double_click
    }

    /// Forget the recent click, e.g. when the pointer leaves the surface or
    /// the surface's row disappears.
    pub fn clear(&mut self) {
        self.recent = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_clicks_on_the_same_subject_inside_the_window_double_click() {
        let mut detector = DoubleClick::new();
        let t0 = Instant::now();
        assert!(!detector.note_click("layer-1", t0));
        assert!(detector.note_click("layer-1", t0 + Duration::from_millis(100)));
    }

    #[test]
    fn clicks_outside_the_window_or_on_other_subjects_stay_single() {
        let mut detector = DoubleClick::new();
        let t0 = Instant::now();
        assert!(!detector.note_click("layer-1", t0));
        assert!(!detector.note_click("layer-1", t0 + Duration::from_millis(351)));
        assert!(!detector.note_click("layer-2", t0 + Duration::from_millis(360)));
    }

    #[test]
    fn triple_click_reads_as_double_then_double() {
        let mut detector = DoubleClick::new();
        let t0 = Instant::now();
        assert!(!detector.note_click("layer-1", t0));
        assert!(detector.note_click("layer-1", t0 + Duration::from_millis(50)));
        assert!(detector.note_click("layer-1", t0 + Duration::from_millis(100)));
    }

    #[test]
    fn tuple_subjects_compare_field_by_field_like_the_old_struct_checks() {
        #[derive(Debug, Clone, PartialEq, Eq)]
        enum Piece {
            Single,
            Center,
        }
        type Subject = (char, u32, Piece);
        let mut detector = DoubleClick::new();
        let t0 = Instant::now();
        let left: Subject = ('L', 7, Piece::Single);
        assert!(!detector.note_click(left.clone(), t0));
        assert!(detector.note_click(left, t0 + Duration::from_millis(10)));
        assert!(!detector.note_click(('R', 7, Piece::Single), t0 + Duration::from_millis(20)));
    }

    #[test]
    fn clear_forgets_the_recent_click() {
        let mut detector = DoubleClick::new();
        let t0 = Instant::now();
        assert!(!detector.note_click("layer-1", t0));
        detector.clear();
        assert!(!detector.note_click("layer-1", t0 + Duration::from_millis(10)));
    }

    #[test]
    fn custom_windows_honor_the_given_duration() {
        let mut detector = DoubleClick::<String>::with_window(Duration::from_millis(500));
        let t0 = Instant::now();
        assert!(!detector.note_click("a".to_string(), t0));
        assert!(detector.note_click("a".to_string(), t0 + Duration::from_millis(450)));
    }
}
