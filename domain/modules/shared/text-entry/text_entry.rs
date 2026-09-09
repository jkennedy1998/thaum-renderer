//! Shared module-space text-entry field: one editable single-line draft with
//! focus, commit, and clipboard-paste intake. Any module that needs the user
//! to type one short value (session join codes, name edits, future fields)
//! owns one of these instead of re-rolling key-capture draft handling.
//!
//! The component is pure state + key logic — it draws nothing and owns no
//! rect. The owning module decides where the field renders and routes the
//! registry's key-capture seam (`Module::on_key_capture`) into
//! `handle_key_label`. Clipboard reading stays orchestration-owned: the
//! field only flags `take_paste_request()` when its paste surface is
//! activated and accepts the text back through `insert_text`. This keeps
//! OS-clipboard timing policy in exactly one layer per app, the same rule
//! the session panel's copy-invite already follows.
//!
//! Key labels mirror the existing entrypoint key-label map (`ENTER`,
//! `ESCAPE`, `BACKSPACE`, `SPACE`, single glyph labels). Unknown labels are
//! consumed silently while focused so stray keys never leak into bindings.

/// One focused-or-not single-line text draft. `Default` is unfocused and
/// empty, so panel state structs can embed it directly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TextEntryField {
    focused: bool,
    draft: String,
    max_chars: usize,
    placeholder: String,
    commit: Option<String>,
    paste_requested: bool,
}

impl TextEntryField {
    /// A field with a character cap and idle placeholder text.
    pub fn new(max_chars: usize, placeholder: impl Into<String>) -> Self {
        Self {
            max_chars,
            placeholder: placeholder.into(),
            ..Self::default()
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn is_empty(&self) -> bool {
        self.draft.is_empty()
    }

    /// Gives the field the keyboard. Any prior commit clears — focus always
    /// starts a fresh edit.
    pub fn focus(&mut self) {
        self.focused = true;
        self.commit = None;
    }

    /// Takes the keyboard away and drops the draft (escape semantics; a
    /// committed value already left through `take_commit`).
    pub fn blur(&mut self) {
        self.focused = false;
        self.draft.clear();
    }

    /// The draft with the focused cursor mark, the placeholder when idle
    /// and empty, or the bare cursor on a fresh focused edit. The owning
    /// module renders this verbatim.
    pub fn display(&self) -> String {
        if self.draft.is_empty() {
            if self.focused {
                return "_".to_string();
            }
            return self.placeholder.clone();
        }
        if self.focused {
            return format!("{}_", self.draft);
        }
        self.draft.clone()
    }

    /// Handles one key label while focused. Returns whether the label was
    /// consumed — always true when focused (focused fields own the keyboard),
    /// always false when not.
    pub fn handle_key_label(&mut self, label: &str) -> bool {
        if !self.focused {
            return false;
        }
        match label {
            "ENTER" => {
                self.focused = false;
                if self.draft.trim().is_empty() {
                    self.draft.clear();
                } else {
                    self.commit = Some(self.draft.trim().to_string());
                }
                self.draft.clear();
            }
            "ESCAPE" => self.blur(),
            "BACKSPACE" | "DELETE" => {
                self.draft.pop();
            }
            "SPACE" => self.push_char(' '),
            single if single.chars().count() == 1 => self.push_str(single),
            _ => {}
        }
        true
    }

    /// Orchestrations apply clipboard text here after a paste request. The
    /// field accepts it whether focused or not (paste is intent, not
    /// typing), and the field gains focus so the user sees and can fix the
    /// pasted draft.
    pub fn insert_text(&mut self, text: &str) {
        self.focused = true;
        for glyph in text.chars().filter(|glyph| !glyph.is_control()) {
            self.push_char(glyph);
        }
    }

    /// The committed value, if Enter went through since the last take.
    pub fn take_commit(&mut self) -> Option<String> {
        self.commit.take()
    }

    /// True once since the last take when the paste surface was activated.
    pub fn take_paste_request(&mut self) -> bool {
        std::mem::take(&mut self.paste_requested)
    }

    /// Flags a paste request — the owning module calls this from its click
    /// handling for the paste surface, keeping the field the only place
    /// paste intent lives.
    pub fn request_paste(&mut self) {
        self.paste_requested = true;
    }

    fn push_char(&mut self, glyph: char) {
        if self.draft.chars().count() < self.max_chars {
            self.draft.push(glyph);
        }
    }

    fn push_str(&mut self, text: &str) {
        for glyph in text.chars() {
            self.push_char(glyph);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unfocused_field_never_consumes_keys() {
        let mut field = TextEntryField::new(21, "code");
        assert!(!field.handle_key_label("A"));
        assert_eq!(field.display(), "code");
    }

    #[test]
    fn typing_commits_on_enter_and_clears_on_escape() {
        let mut field = TextEntryField::new(21, "code");
        field.focus();
        for label in ["A", "B", "C"] {
            assert!(field.handle_key_label(label));
        }
        assert_eq!(field.display(), "ABC_");
        assert!(field.handle_key_label("ENTER"));
        assert_eq!(field.take_commit().as_deref(), Some("ABC"));
        assert!(field.take_commit().is_none());
        assert!(!field.is_focused());

        field.focus();
        assert!(field.handle_key_label("X"));
        assert!(field.handle_key_label("ESCAPE"));
        assert!(field.take_commit().is_none());
        assert!(field.is_empty());
        assert!(!field.is_focused());
    }

    #[test]
    fn backspace_space_and_cap_respect_the_limit() {
        let mut field = TextEntryField::new(2, "code");
        field.focus();
        assert!(field.handle_key_label("A"));
        assert!(field.handle_key_label("B"));
        assert!(field.handle_key_label("C"));
        assert_eq!(field.display(), "AB_");
        assert!(field.handle_key_label("BACKSPACE"));
        assert!(field.handle_key_label("SPACE"));
        assert_eq!(field.display(), "A _");
        // Empty commits stay empty: no action comes out of Enter on a draft
        // that is only whitespace.
        field.handle_key_label("BACKSPACE");
        field.handle_key_label("BACKSPACE");
        field.handle_key_label("SPACE");
        field.handle_key_label("ENTER");
        assert!(field.take_commit().is_none());
    }

    #[test]
    fn paste_request_routes_through_take_and_insert_fills_the_draft() {
        let mut field = TextEntryField::new(21, "code");
        field.request_paste();
        assert!(field.take_paste_request());
        assert!(!field.take_paste_request());

        field.insert_text("ab\nc1");
        assert!(field.is_focused());
        assert_eq!(field.display(), "abc1_");
        assert!(field.handle_key_label("ENTER"));
        assert_eq!(field.take_commit().as_deref(), Some("abc1"));
    }

    #[test]
    fn refocus_after_commit_starts_a_fresh_edit() {
        let mut field = TextEntryField::new(21, "code");
        field.focus();
        field.insert_text("old");
        field.handle_key_label("ENTER");
        assert_eq!(field.take_commit().as_deref(), Some("old"));

        field.focus();
        assert_eq!(field.display(), "_");
        assert!(field.take_commit().is_none());
    }
}
