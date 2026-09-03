use std::collections::HashSet;

use crate::controls::RawInput;

/// Where one raw input goes while a typing session owns the input surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypingRoute {
    /// Declared reserved at `begin`: stays live outside the session so the
    /// consuming program dispatches it through its normal paths.
    Reserved,
    /// Belongs to the focused typing session: the owner feeds it to its own
    /// text-entry state and must not let it reach any other consumer.
    Owned,
    /// Suppressed while typing mode is active: no consumer sees it.
    Suppressed,
}

/// Renderer-owned input-focus gate for typing sessions (text tools, text
/// fields, any modal that captures the keyboard).
///
/// The seam owns only the routing decision: which inputs stay reserved
/// outside the session, which belong to the session, and which are
/// suppressed. It owns no key semantics — translating owned keys into text
/// entry stays with the consuming program, and the reserved set is declared
/// at `begin` rather than defaulted here, matching the controls rule that
/// no default bindings ship with the renderer.
///
/// While active, every unreserved key routes as `Owned` and pointer/wheel
/// inputs route as `Suppressed`, so clicks and scroll cannot reach modules
/// or the canvas behind the session's back. An app may declare pointer or
/// wheel inputs reserved too if its session needs them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypingMode {
    active: bool,
    reserved: HashSet<RawInput>,
}

impl TypingMode {
    /// Begin one typing session that owns the input surface. `reserved`
    /// declares the inputs that stay live outside the session; beginning
    /// again replaces the previous session's reserved set.
    pub fn begin(&mut self, reserved: impl IntoIterator<Item = RawInput>) {
        self.active = true;
        self.reserved = reserved.into_iter().collect();
    }

    /// End the typing session and release the input surface.
    pub fn end(&mut self) {
        *self = Self::default();
    }

    /// Whether a typing session currently owns the input surface.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// The reserved set declared at `begin`.
    pub fn reserved(&self) -> &HashSet<RawInput> {
        &self.reserved
    }

    /// Route one input while a session is active. Callers gate on
    /// `is_active` before routing; this decision is only meaningful then.
    pub fn route(&self, input: &RawInput) -> TypingRoute {
        if self.reserved.contains(input) {
            TypingRoute::Reserved
        } else if matches!(input, RawInput::Key(_)) {
            TypingRoute::Owned
        } else {
            TypingRoute::Suppressed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(label: &str) -> RawInput {
        RawInput::Key(label.to_string())
    }

    #[test]
    fn reserved_inputs_stay_live_while_the_session_owns_the_keyboard() {
        let mut mode = TypingMode::default();
        mode.begin([key("NUMPAD4"), key("NUMPAD6")]);

        assert_eq!(mode.route(&key("NUMPAD4")), TypingRoute::Reserved);
        assert_eq!(mode.route(&key("NUMPAD6")), TypingRoute::Reserved);
    }

    #[test]
    fn unreserved_keys_belong_to_the_session() {
        let mut mode = TypingMode::default();
        mode.begin([key("NUMPAD4")]);

        assert_eq!(mode.route(&key("A")), TypingRoute::Owned);
        assert_eq!(mode.route(&key("ENTER")), TypingRoute::Owned);
    }

    #[test]
    fn pointer_and_wheel_input_is_suppressed_unless_declared_reserved() {
        let mut mode = TypingMode::default();
        mode.begin([RawInput::MouseButton(0)]);

        assert_eq!(mode.route(&RawInput::MouseWheelUp), TypingRoute::Suppressed);
        assert_eq!(
            mode.route(&RawInput::MouseWheelDown),
            TypingRoute::Suppressed
        );
        assert_eq!(
            mode.route(&RawInput::MouseButton(1)),
            TypingRoute::Suppressed
        );
        assert_eq!(mode.route(&RawInput::MouseButton(0)), TypingRoute::Reserved);
    }

    #[test]
    fn ending_the_session_clears_activity_and_the_reserved_set() {
        let mut mode = TypingMode::default();
        mode.begin([key("NUMPAD4")]);
        assert!(mode.is_active());

        mode.end();
        assert!(!mode.is_active());
        assert!(mode.reserved().is_empty());
        assert_eq!(mode, TypingMode::default());
    }

    #[test]
    fn beginning_again_replaces_the_previous_sessions_reserved_set() {
        let mut mode = TypingMode::default();
        mode.begin([key("NUMPAD4")]);
        mode.begin([key("NUMPAD8")]);

        assert_eq!(mode.route(&key("NUMPAD8")), TypingRoute::Reserved);
        assert_eq!(mode.route(&key("NUMPAD4")), TypingRoute::Owned);
    }
}
