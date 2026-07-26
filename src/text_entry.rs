//! Soft-keyboard-backed text entry, used for the post-game-over high-score
//! name prompt. Typed text and the keyboard's action-key press arrive as
//! ordinary input events (`InputEvent::TextEvent` / `TextAction`), forwarded
//! here as `input::Action::TextChanged`/`TextSubmitted` — see `src/input.rs`.

use android_activity::input::{ImeOptions, InputType, TextInputAction, TextInputState};
use android_activity::AndroidApp;

/// Typed names longer than this are truncated as they arrive.
const MAX_LEN: usize = 12;

pub struct NameEntry {
    active: bool,
    text: String,
}

impl NameEntry {
    pub fn new() -> Self {
        NameEntry { active: false, text: String::new() }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    /// Shows the soft keyboard and starts capturing text.
    pub fn activate(&mut self, app: &AndroidApp) {
        self.active = true;
        self.text.clear();
        app.set_text_input_state(TextInputState::default());
        app.set_ime_editor_info(
            InputType::TYPE_CLASS_TEXT | InputType::TYPE_TEXT_FLAG_CAP_CHARACTERS,
            TextInputAction::Done,
            ImeOptions::empty(),
        );
        app.show_soft_input(true);
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text.chars().take(MAX_LEN).collect();
    }

    /// Hides the keyboard and returns the captured text, ending entry.
    pub fn submit(&mut self, app: &AndroidApp) -> String {
        self.active = false;
        app.hide_soft_input(true);
        std::mem::take(&mut self.text)
    }
}
