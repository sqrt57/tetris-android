pub mod game;

#[cfg(target_os = "android")]
mod input;
#[cfg(target_os = "android")]
mod lifecycle;
#[cfg(target_os = "android")]
mod renderer;
#[cfg(target_os = "android")]
mod text_entry;

#[cfg(target_os = "android")]
use android_activity::{AndroidApp, MainEvent, PollEvent};
#[cfg(target_os = "android")]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    log::info!("tetris-android starting");

    let mut state = lifecycle::AppState::new();
    let mut touch = input::TouchInput::new();
    let mut game = game::Game::new(0x5EED);
    let mut name_entry = text_entry::NameEntry::new();
    let mut high_score: u32 = 0;
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
                            log::info!("window resized: {width}x{height}");
                            state.on_window_resized(width, height);
                        }
                    }
                    MainEvent::ConfigChanged { .. } => {
                        // Rotation/config changes are handled in-process (see
                        // `configChanges` in AndroidManifest.xml) rather than
                        // recreating the Activity, so `game`/`name_entry` state
                        // survives untouched. GameActivity is expected to also
                        // fire WindowResized when the surface itself changes
                        // size, but re-sync here too as a defensive fallback.
                        if let Some(window) = app.native_window() {
                            let width = window.width().max(0) as u32;
                            let height = window.height().max(0) as u32;
                            log::info!("config changed, window now {width}x{height}");
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

        for action in touch.poll(&app, state.width()) {
            match action {
                input::Action::MoveLeft => {
                    game.move_by(-1, 0);
                }
                input::Action::MoveRight => {
                    game.move_by(1, 0);
                }
                input::Action::RotateClockwise => {
                    game.rotate_cw();
                }
                input::Action::SoftDrop => {
                    game.move_by(0, 1);
                }
                input::Action::HardDrop => {
                    game.hard_drop();
                }
                input::Action::TextChanged(text) => {
                    name_entry.set_text(text);
                }
                input::Action::TextSubmitted => {
                    if name_entry.is_active() {
                        let name = name_entry.submit(&app);
                        if game.score > high_score {
                            high_score = game.score;
                            log::info!("New high score: {high_score} ({name})");
                        } else {
                            log::info!("Name entered: {name} (score {})", game.score);
                        }
                        game = game::Game::new(seed_from_clock());
                        last_tick = Instant::now();
                    }
                }
            }
        }

        if game.game_over && !name_entry.is_active() {
            log::info!("game over (score {}), showing keyboard for name entry", game.score);
            name_entry.activate(&app);
        }

        if !game.game_over && last_tick.elapsed() >= tick_interval {
            game.tick();
            last_tick = Instant::now();
        }

        let name_entry_chars = name_entry.is_active().then(|| name_entry.text().chars().count());
        state.render(&game, name_entry_chars);
    }
}

#[cfg(target_os = "android")]
fn seed_from_clock() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1)
}
