//! Tool-assisted input (TAI) runner for the controls domain.
//!
//! A TAI is a small JSON script of breath-timed synthetic inputs plus the
//! named-action fires it expects. The runner replays the timeline through an
//! `ActionBindingMap` and reports which actions fired on which breath and any
//! expectation misses. Scripts live in `individuals/`; `template/` is the
//! copyable starting point; `individuals/registry.json` is the growable list.
//!
//! This owns the script format and the replay seam only. What an action means,
//! app-level assertions beyond action fires, and wiring into a live window are
//! consuming-program concerns.

use serde::Deserialize;
use std::collections::BTreeMap;

use super::{ActionBindingMap, RawInput};

/// One synthetic raw input as encoded in a TAI script.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaiInput {
    Key { code: String },
    MouseButton { button: u8 },
    WheelUp,
    WheelDown,
}

impl From<&TaiInput> for RawInput {
    fn from(input: &TaiInput) -> Self {
        match input {
            TaiInput::Key { code } => RawInput::Key(code.clone()),
            TaiInput::MouseButton { button } => RawInput::MouseButton(*button),
            TaiInput::WheelUp => RawInput::MouseWheelUp,
            TaiInput::WheelDown => RawInput::MouseWheelDown,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TaiScriptFile {
    id: String,
    #[serde(default)]
    description: String,
    actions: Vec<TaiStepFile>,
    #[serde(default)]
    expect: Vec<TaiExpectFile>,
}

#[derive(Debug, Deserialize)]
struct TaiStepFile {
    at: u64,
    input: TaiInput,
}

#[derive(Debug, Deserialize)]
struct TaiExpectFile {
    at: u64,
    action: String,
}

/// One expected action fire: `action` should fire on breath `at`.
#[derive(Debug, Clone, PartialEq)]
pub struct TaiExpectation {
    pub at: u64,
    pub action: String,
}

/// A parsed TAI script: breath-indexed input steps plus expected action fires.
#[derive(Debug, Clone)]
pub struct TaiScript {
    pub id: String,
    pub description: String,
    timeline: BTreeMap<u64, Vec<TaiInput>>,
    expectations: Vec<TaiExpectation>,
}

impl TaiScript {
    /// Parse a script from raw JSON text.
    pub fn parse(json: &str) -> Result<Self, String> {
        let file: TaiScriptFile =
            serde_json::from_str(json).map_err(|e| format!("tai_script_invalid: {e}"))?;
        if file.id.trim().is_empty() {
            return Err("tai_script_missing_id".to_string());
        }
        if file.actions.is_empty() {
            return Err("tai_script_missing_actions".to_string());
        }
        let mut timeline: BTreeMap<u64, Vec<TaiInput>> = BTreeMap::new();
        for step in file.actions {
            timeline.entry(step.at).or_default().push(step.input);
        }
        Ok(Self {
            id: file.id,
            description: file.description,
            timeline,
            expectations: file
                .expect
                .into_iter()
                .map(|e| TaiExpectation { at: e.at, action: e.action })
                .collect(),
        })
    }

    /// Replay the timeline through `bindings` and check every expectation.
    /// Inputs at breaths >= `total_breaths` never fire and fail their expectations.
    pub fn run(&self, bindings: &ActionBindingMap, total_breaths: u64) -> TaiRunReport {
        let mut fired: Vec<(u64, String)> = Vec::new();
        for (breath, inputs) in &self.timeline {
            if *breath >= total_breaths {
                break;
            }
            for input in inputs {
                for action in bindings.actions_for(input.into()) {
                    fired.push((*breath, action.0.clone()));
                }
            }
        }

        let mut expectation_misses: Vec<TaiExpectation> = Vec::new();
        for expectation in &self.expectations {
            let hit = fired
                .iter()
                .any(|(at, action)| *at == expectation.at && *action == expectation.action);
            if !hit {
                expectation_misses.push(expectation.clone());
            }
        }

        TaiRunReport {
            script_id: self.id.clone(),
            fired,
            expectation_misses,
        }
    }
}

/// Result of one TAI replay: everything that fired plus expectation misses.
#[derive(Debug, Clone, PartialEq)]
pub struct TaiRunReport {
    pub script_id: String,
    pub fired: Vec<(u64, String)>,
    pub expectation_misses: Vec<TaiExpectation>,
}

impl TaiRunReport {
    pub fn passed(&self) -> bool {
        self.expectation_misses.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::ActionName;

    fn bindings() -> ActionBindingMap {
        let mut map = ActionBindingMap::new();
        map.bind(
            ActionName::new("tai_pan_left"),
            RawInput::Key("A".to_string()),
        );
        map.bind(
            ActionName::new("tai_primary_click"),
            RawInput::MouseButton(0),
        );
        map.bind(ActionName::new("tai_zoom_in"), RawInput::MouseWheelUp);
        map.bind(
            ActionName::new("example_action"),
            RawInput::Key("A".to_string()),
        );
        map
    }

    #[test]
    fn template_parses_and_runs_against_declared_bindings() {
        let script =
            TaiScript::parse(include_str!("template/script.json")).expect("template parses");
        let report = script.run(&bindings(), 16);
        assert!(report.passed(), "misses: {:?}", report.expectation_misses);
    }

    #[test]
    fn missed_expectation_fails_the_run() {
        let script = TaiScript::parse(
            r#"{
                "id": "regression",
                "actions": [{ "at": 1, "input": { "type": "key", "code": "A" } }],
                "expect": [{ "at": 2, "action": "tai_pan_left" }]
            }"#,
        )
        .expect("parses");
        let report = script.run(&bindings(), 16);
        assert!(!report.passed());
        assert_eq!(
            report.expectation_misses,
            vec![TaiExpectation { at: 2, action: "tai_pan_left".to_string() }]
        );
    }

    #[test]
    fn input_after_total_breaths_never_fires() {
        let script = TaiScript::parse(
            r#"{
                "id": "late",
                "actions": [{ "at": 9, "input": { "type": "key", "code": "A" } }],
                "expect": [{ "at": 9, "action": "tai_pan_left" }]
            }"#,
        )
        .expect("parses");
        let report = script.run(&bindings(), 10);
        assert!(report.passed());
        let script = TaiScript::parse(
            r#"{
                "id": "too_late",
                "actions": [{ "at": 10, "input": { "type": "key", "code": "A" } }],
                "expect": [{ "at": 10, "action": "tai_pan_left" }]
            }"#,
        )
        .expect("parses");
        let report = script.run(&bindings(), 10);
        assert!(!report.passed());
    }

    #[test]
    fn unbound_input_fires_nothing() {
        let script = TaiScript::parse(
            r#"{
                "id": "unbound",
                "actions": [{ "at": 1, "input": { "type": "key", "code": "Q" } }]
            }"#,
        )
        .expect("parses");
        let report = script.run(&bindings(), 16);
        assert!(report.passed());
        assert!(report.fired.is_empty());
    }
}
