//! Touch input mapping — not wired up yet.
//!
//! GameActivity delivers input via `AndroidApp::input_events_iter()`. This is left
//! as a stub until the render loop has a board to react to; wiring it prematurely
//! risks guessing the wrong android-activity input API version and breaking the
//! build before it's ever exercised.

#![allow(dead_code)]

pub enum Action {
    MoveLeft,
    MoveRight,
    RotateClockwise,
    SoftDrop,
    HardDrop,
}
