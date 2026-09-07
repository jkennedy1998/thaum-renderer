//! User-owned control-binding profile seam.
//!
//! The one source of truth for *effective* control bindings: a program
//! declares default bindings (its `ActionBindingMap`), a user profile stores
//! only overrides, and [`effective_bindings`] merges the two. Live input
//! dispatch, the rebind UI, and TAI-style harnesses all resolve through this
//! merge instead of holding private copies.
//!
//! Overrides-only by design: the profile never restates defaults, so newly
//! declared actions automatically pick up their declared binding until the
//! user changes them.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{ActionBindingMap, ActionName, RawInput};

/// Per-user override profile over a program's declared default bindings.
/// `Some(input)` rebinds the action; `None` unbinds (disables) it; an absent
/// entry keeps the declared default. Stored per user, never per document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlsProfile {
    pub version: u32,
    #[serde(default)]
    pub bindings: BTreeMap<String, Option<RawInput>>,
}

impl ControlsProfile {
    pub fn new() -> Self {
        Self {
            version: 1,
            bindings: BTreeMap::new(),
        }
    }

    pub fn set_override(&mut self, action: &ActionName, binding: Option<RawInput>) {
        self.bindings.insert(action.0.clone(), binding);
    }

    /// The stored override for one action, if any.
    pub fn override_for(&self, action: &ActionName) -> Option<&Option<RawInput>> {
        self.bindings.get(&action.0)
    }
}

impl Default for ControlsProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Merge declared defaults with user overrides into the effective binding map.
/// This is the single source of truth every consumer resolves bindings from.
pub fn effective_bindings(
    defaults: &ActionBindingMap,
    profile: &ControlsProfile,
) -> ActionBindingMap {
    let mut effective = defaults.clone();
    for (name, binding) in &profile.bindings {
        let action = ActionName::new(name);
        match binding {
            Some(input) => effective.rebind(action, input.clone()),
            None => effective.unbind(action),
        }
    }
    effective
}

/// Every other action whose effective bindings share any raw input with the
/// given action. Empty when the action is unbound or exclusive.
pub fn conflicting_actions(effective: &ActionBindingMap, action: &ActionName) -> Vec<ActionName> {
    let own = effective.bindings_for(action);
    if own.is_empty() {
        return Vec::new();
    }
    let mut conflicts = Vec::new();
    for other in effective.actions() {
        if other == action {
            continue;
        }
        if effective
            .bindings_for(other)
            .iter()
            .any(|input| own.contains(input))
        {
            conflicts.push(other.clone());
        }
    }
    conflicts
}

/// Human-readable label for one raw input, e.g. `P`, `MOUSE 1`, `WHEEL UP`.
pub fn format_raw_input(input: &RawInput) -> String {
    match input {
        RawInput::Key(code) => code.clone(),
        RawInput::MouseButton(button) => format!("MOUSE {button}"),
        RawInput::MouseWheelUp => "WHEEL UP".to_string(),
        RawInput::MouseWheelDown => "WHEEL DOWN".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> ActionBindingMap {
        let mut map = ActionBindingMap::new();
        map.bind(ActionName::new("a"), RawInput::Key("A".to_string()));
        map.bind(ActionName::new("b"), RawInput::Key("B".to_string()));
        map.bind(ActionName::new("wheel"), RawInput::MouseWheelUp);
        map
    }

    #[test]
    fn empty_profile_keeps_every_default() {
        let effective = effective_bindings(&defaults(), &ControlsProfile::new());
        assert_eq!(
            effective.bindings_for(&ActionName::new("a")),
            &[RawInput::Key("A".to_string())]
        );
        assert_eq!(
            effective.bindings_for(&ActionName::new("wheel")),
            &[RawInput::MouseWheelUp]
        );
    }

    #[test]
    fn rebind_override_replaces_default() {
        let mut profile = ControlsProfile::new();
        profile.set_override(&ActionName::new("a"), Some(RawInput::Key("Q".to_string())));
        let effective = effective_bindings(&defaults(), &profile);
        assert_eq!(
            effective.bindings_for(&ActionName::new("a")),
            &[RawInput::Key("Q".to_string())]
        );
        assert_eq!(
            effective.bindings_for(&ActionName::new("b")),
            &[RawInput::Key("B".to_string())]
        );
    }

    #[test]
    fn none_override_unbinds_the_action() {
        let mut profile = ControlsProfile::new();
        profile.set_override(&ActionName::new("b"), None);
        let effective = effective_bindings(&defaults(), &profile);
        assert!(effective.bindings_for(&ActionName::new("b")).is_empty());
        assert_eq!(effective.actions().count(), 3, "action stays declared");
    }

    #[test]
    fn unbound_actions_report_no_conflicts() {
        let mut profile = ControlsProfile::new();
        profile.set_override(&ActionName::new("b"), Some(RawInput::Key("A".to_string())));
        let effective = effective_bindings(&defaults(), &profile);
        assert_eq!(
            conflicting_actions(&effective, &ActionName::new("b")),
            vec![ActionName::new("a")]
        );
        assert_eq!(
            conflicting_actions(&effective, &ActionName::new("a")),
            vec![ActionName::new("b")]
        );
        assert!(conflicting_actions(&effective, &ActionName::new("wheel")).is_empty());
    }

    #[test]
    fn profile_serializes_round_trip_with_overrides_only() {
        let mut profile = ControlsProfile::new();
        profile.set_override(&ActionName::new("a"), Some(RawInput::Key("Q".to_string())));
        profile.set_override(&ActionName::new("b"), None);
        let raw = serde_json::to_string(&profile).expect("serializes");
        let parsed: ControlsProfile = serde_json::from_str(&raw).expect("parses");
        assert_eq!(parsed, profile);
    }

    #[test]
    fn raw_input_labels_are_human_readable() {
        assert_eq!(format_raw_input(&RawInput::Key("P".to_string())), "P");
        assert_eq!(format_raw_input(&RawInput::MouseButton(2)), "MOUSE 2");
        assert_eq!(format_raw_input(&RawInput::MouseWheelUp), "WHEEL UP");
        assert_eq!(format_raw_input(&RawInput::MouseWheelDown), "WHEEL DOWN");
    }
}
