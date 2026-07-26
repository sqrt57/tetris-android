//! Touch input: classifies raw down/up gestures into game actions.
//!
//! Zone tap (small movement): left third = move left, right third = move
//! right, middle third = rotate. Swipe down: short = soft drop, long = hard
//! drop. Everything else (horizontal swipes, multi-touch) is ignored — this
//! is deliberately the simplest scheme that covers all five actions.
//!
//! Also forwards IME text events (soft-keyboard input, see
//! `src/text_entry.rs`): `android-activity` delivers those through the same
//! lending input-event iterator as touch/key events, and only one consumer
//! per frame may drain it, so this is that one place.

use android_activity::input::{InputEvent, MotionAction};
use android_activity::{AndroidApp, InputStatus};

pub enum Action {
    MoveLeft,
    MoveRight,
    RotateClockwise,
    SoftDrop,
    HardDrop,
    /// The soft keyboard's text state changed; carries the full current text.
    TextChanged(String),
    /// The soft keyboard's action key (Done) was pressed.
    TextSubmitted,
}

const TAP_MAX_MOVEMENT: f32 = 24.0;
const HARD_DROP_MIN_DISTANCE: f32 = 220.0;

pub struct TouchInput {
    /// (x, y) of the primary pointer's last ACTION_DOWN.
    down: Option<(f32, f32)>,
}

impl TouchInput {
    pub fn new() -> Self {
        TouchInput { down: None }
    }

    /// Drains all buffered input events and returns the game actions they produced.
    /// `screen_width` (in the same pixel space as touch coordinates) is needed to
    /// tell left/middle/right zone taps apart.
    pub fn poll(&mut self, app: &AndroidApp, screen_width: u32) -> Vec<Action> {
        let mut actions = Vec::new();

        let mut iter = match app.input_events_iter() {
            Ok(iter) => iter,
            Err(err) => {
                log::error!("input_events_iter failed: {err}");
                return actions;
            }
        };

        loop {
            let had_event = iter.next(|event| match event {
                InputEvent::MotionEvent(motion_event) => {
                    let pointer = motion_event.pointer_at_index(motion_event.pointer_index());
                    let x = pointer.x();
                    let y = pointer.y();
                    match motion_event.action() {
                        MotionAction::Down => {
                            self.down = Some((x, y));
                        }
                        MotionAction::Up => {
                            if let Some((sx, sy)) = self.down.take() {
                                if let Some(action) = classify(sx, sy, x, y, screen_width) {
                                    actions.push(action);
                                }
                            }
                        }
                        MotionAction::Cancel => {
                            self.down = None;
                        }
                        _ => {}
                    }
                    InputStatus::Handled
                }
                InputEvent::KeyEvent(_) => InputStatus::Unhandled,
                InputEvent::TextEvent(state) => {
                    actions.push(Action::TextChanged(state.text.clone()));
                    InputStatus::Handled
                }
                InputEvent::TextAction(_) => {
                    actions.push(Action::TextSubmitted);
                    InputStatus::Handled
                }
                _ => InputStatus::Unhandled,
            });
            if !had_event {
                break;
            }
        }

        actions
    }
}

fn classify(start_x: f32, start_y: f32, end_x: f32, end_y: f32, screen_width: u32) -> Option<Action> {
    let dx = end_x - start_x;
    let dy = end_y - start_y;

    if dx.abs() <= TAP_MAX_MOVEMENT && dy.abs() <= TAP_MAX_MOVEMENT {
        let third = screen_width as f32 / 3.0;
        return Some(if start_x < third {
            Action::MoveLeft
        } else if start_x > third * 2.0 {
            Action::MoveRight
        } else {
            Action::RotateClockwise
        });
    }

    if dy > 0.0 && dy.abs() > dx.abs() {
        return Some(if dy >= HARD_DROP_MIN_DISTANCE {
            Action::HardDrop
        } else {
            Action::SoftDrop
        });
    }

    None
}
