//! Owns the renderer across Android's surface-destroyed/recreated lifecycle.
//! The GPU surface and everything built from it must be dropped before the
//! ANativeWindow it was created from, and rebuilt when a new window arrives.

use crate::game::Game;
use crate::renderer::Renderer;
use ndk::native_window::NativeWindow;

pub struct AppState {
    renderer: Option<Renderer>,
}

impl AppState {
    pub fn new() -> Self {
        AppState { renderer: None }
    }

    pub fn on_window_created(&mut self, window: NativeWindow) {
        let width = window.width().max(0) as u32;
        let height = window.height().max(0) as u32;
        match Renderer::new(window, width, height) {
            Ok(renderer) => self.renderer = Some(renderer),
            Err(err) => log::error!("failed to create renderer: {err}"),
        }
    }

    pub fn on_window_destroyed(&mut self) {
        self.renderer = None;
    }

    pub fn on_window_resized(&mut self, width: u32, height: u32) {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height);
        }
    }

    pub fn render(&mut self, game: &Game, name_entry_text: Option<&str>) {
        if let Some(renderer) = &mut self.renderer {
            renderer.render(game, name_entry_text);
        }
    }

    /// Current surface width in pixels, or 0 while there is no window (used to
    /// classify touch zones; a spurious 0-width frame just drops the input).
    pub fn width(&self) -> u32 {
        self.renderer.as_ref().map_or(0, |r| r.width())
    }
}
