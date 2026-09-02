use std::collections::HashMap;

/// A raw physical input, independent of what any program has bound it to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RawInput {
    Key(&'static str),
    MouseButton(u8),
    MouseWheelUp,
    MouseWheelDown,
}

/// A pressure sample from a pressure-sensitive input device, 0.0..=1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PressureSample {
    pub pressure: f32,
}

impl PressureSample {
    pub fn clamped(pressure: f32) -> Self {
        Self {
            pressure: pressure.clamp(0.0, 1.0),
        }
    }
}

/// One program-declared named action, e.g. "camera_pan_left" or "brush_stroke".
/// The renderer never defines these itself; programs declare and bind their own.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ActionName(pub String);

impl ActionName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }
}

/// The remappable binding table between named actions and raw inputs.
/// Ships with no default bindings; a consuming program registers its own
/// actions and binds them before this map can resolve anything.
#[derive(Debug, Clone, Default)]
pub struct ActionBindingMap {
    bindings: HashMap<ActionName, Vec<RawInput>>,
}

impl ActionBindingMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare a named action with no bound inputs yet.
    pub fn register_action(&mut self, action: ActionName) {
        self.bindings.entry(action).or_default();
    }

    /// Bind one raw input to a named action, in addition to any existing bindings.
    /// Registers the action if it was not already declared.
    pub fn bind(&mut self, action: ActionName, input: RawInput) {
        let inputs = self.bindings.entry(action).or_default();
        if !inputs.contains(&input) {
            inputs.push(input);
        }
    }

    /// Replace all bindings for a named action with exactly one raw input (remap).
    pub fn rebind(&mut self, action: ActionName, input: RawInput) {
        self.bindings.insert(action, vec![input]);
    }

    pub fn bindings_for(&self, action: &ActionName) -> &[RawInput] {
        self.bindings.get(action).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Resolve a raw input back to every named action it is currently bound to.
    pub fn actions_for(&self, input: RawInput) -> Vec<&ActionName> {
        self.bindings
            .iter()
            .filter(|(_, inputs)| inputs.contains(&input))
            .map(|(action, _)| action)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ships_with_no_bindings_until_a_program_declares_them() {
        let map = ActionBindingMap::new();
        assert!(map
            .bindings_for(&ActionName::new("camera_pan_left"))
            .is_empty());
    }

    #[test]
    fn bind_resolves_raw_input_to_named_action() {
        let mut map = ActionBindingMap::new();
        let pan_left = ActionName::new("camera_pan_left");
        map.bind(pan_left.clone(), RawInput::Key("A"));

        assert_eq!(map.bindings_for(&pan_left), &[RawInput::Key("A")]);
        assert_eq!(map.actions_for(RawInput::Key("A")), vec![&pan_left]);
    }

    #[test]
    fn rebind_replaces_rather_than_accumulates() {
        let mut map = ActionBindingMap::new();
        let zoom_in = ActionName::new("zoom_in");
        map.bind(zoom_in.clone(), RawInput::Key("-"));
        map.rebind(zoom_in.clone(), RawInput::MouseWheelUp);

        assert_eq!(map.bindings_for(&zoom_in), &[RawInput::MouseWheelUp]);
    }

    #[test]
    fn pressure_sample_clamps_to_unit_range() {
        assert_eq!(PressureSample::clamped(1.5).pressure, 1.0);
        assert_eq!(PressureSample::clamped(-0.2).pressure, 0.0);
    }
}
