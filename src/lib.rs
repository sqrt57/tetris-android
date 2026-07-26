pub mod game;

#[cfg(target_os = "android")]
mod input;
#[cfg(target_os = "android")]
mod lifecycle;
#[cfg(target_os = "android")]
mod renderer;

#[cfg(target_os = "android")]
use android_activity::{AndroidApp, MainEvent, PollEvent};
#[cfg(target_os = "android")]
use std::time::{Duration, Instant};

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    log::info!("tetris-android starting");

    let mut state = lifecycle::AppState::new();
    let mut game = game::Game::new(0x5EED);
    let tick_interval = Duration::from_millis(500);
    let mut last_tick = Instant::now();
    let mut quit = false;

    while !quit {
        app.poll_events(Some(Duration::from_millis(16)), |event| {
            if let PollEvent::Main(main_event) = event {
                match main_event {
                    MainEvent::InitWindow { .. } => {
                        if let Some(window) = app.native_window() {
                            state.on_window_created(window);
                        }
                    }
                    MainEvent::TerminateWindow { .. } => {
                        state.on_window_destroyed();
                    }
                    MainEvent::WindowResized { .. } => {
                        if let Some(window) = app.native_window() {
                            let width = window.width().max(0) as u32;
                            let height = window.height().max(0) as u32;
                            state.on_window_resized(width, height);
                        }
                    }
                    MainEvent::Destroy => {
                        quit = true;
                    }
                    _ => {}
                }
            }
        });

        if last_tick.elapsed() >= tick_interval {
            game.tick();
            last_tick = Instant::now();
        }

        state.render();
    }
}
