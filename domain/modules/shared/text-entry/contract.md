# thaum-renderer/domain/modules/shared/text-entry

## purpose
Own the shared single-line text-entry field every module can embed: focus, draft, commit, and clipboard-paste intake, so modules stop re-rolling key-capture draft handling one panel at a time.

## owns
- the `TextEntryField` state machine: focus/unfocus, keyed draft editing through the existing key-label vocabulary, Enter commit, Escape blur, character cap, placeholder display with cursor mark
- paste-intent flags: `request_paste` (module-side activation) and `take_paste_request`; `insert_text` as the one paste-application entry
- the consume rule: a focused field owns the keyboard (always consumes), an unfocused field never does

## does not own
- any drawing or rect truth — the owning module renders `display()` wherever it wants
- clipboard reading (OS clipboard timing stays orchestration-owned; the field only accepts text back through `insert_text`)
- the key-label map or raw input capture, owned by consuming programs on top of `domain/controls/`
- multi-line or canvas text entry (`thaum-painter/domain/painter-session/text-entry/` is the canvas typing session, a different concept)

## children-encapsulations
- none

## contents
- `text_entry.rs`
  - `TextEntryField` plus inline tests

## dependencies
- none

## exposed interfaces
- `TextEntryField` — `new`/`focus`/`blur`/`display`/`handle_key_label`/`insert_text`/`request_paste`/`take_commit`/`take_paste_request`/`is_focused`/`is_empty`

## interface consumers
- `thaum-painter/domain/modules/individuals/session-panel/` (join-code field, first consumer)
- future module fields (name edits, numeric entries with formatting wrappers)

## artifacts
- none

## tests
- inline `#[cfg(test)]` in `text_entry.rs`
  - light
  - validates unfocused fall-through, commit/escape semantics, whitespace-only commit suppression, character cap, backspace/space, paste request take-once, control-character filtering on paste, insert-gains-focus, and fresh-edit-on-refocus

## data
- none
